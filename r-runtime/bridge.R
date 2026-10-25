# r-runtime/bridge.R
# Persistent Bidirectional JSON-RPC Bridge between Rust and R

suppressPackageStartupMessages({
  library(jsonlite)
})

# Source sibling helpers
script_dir <- tryCatch({
  args <- commandArgs(trailingOnly = FALSE)
  file_arg <- grep("--file=", args, value = TRUE)
  if (length(file_arg) > 0) {
    dirname(normalizePath(sub("--file=", "", file_arg[1])))
  } else if (file.exists("r-runtime/metadata.R")) {
    normalizePath("r-runtime")
  } else {
    getwd()
  }
}, error = function(e) getwd())

source(file.path(script_dir, "metadata.R"))
source(file.path(script_dir, "formatters.R"))

# Default active dataset
.active_dataset_name <- "mtcars"
.active_data <- datasets::mtcars
attr(.active_data, "dataset_name") <- "mtcars"
assign(".active_data", .active_data, envir = .GlobalEnv)

# Alias true and false to prevent errors from lowercase booleans in generated syntax
assign("true", TRUE, envir = .GlobalEnv)
assign("false", FALSE, envir = .GlobalEnv)

# --- UNDO / REDO & DATA SAFETY CRASH RECOVERY SUBSYSTEM ---
.undo_stack <- list()
.redo_stack <- list()
.max_undo_levels <- 30

rrad_write_atomic_checkpoint <- function(df) {
  tryCatch({
    home_dir <- Sys.getenv("HOME")
    if (!nzchar(home_dir)) home_dir <- tempdir()
    cfg_dir <- file.path(home_dir, ".config", "r-rad-studio", "autosave")
    if (!dir.exists(cfg_dir)) {
      dir.create(cfg_dir, recursive = TRUE, showWarnings = FALSE)
    }
    tmp_file <- file.path(cfg_dir, "checkpoint.tmp.rds")
    final_file <- file.path(cfg_dir, "checkpoint.rds")
    saveRDS(df, tmp_file)
    file.rename(tmp_file, final_file)
  }, error = function(e) {
    tryCatch(saveRDS(df, file.path(tempdir(), "rrad_checkpoint.rds")), error = function(e2) {})
  })
}

rrad_push_undo_checkpoint <- function(action_name = "Data Edit") {
  if (!exists(".active_data", envir = .GlobalEnv)) return()
  curr_data <- .GlobalEnv$.active_data

  snapshot <- curr_data
  attr(snapshot, "dataset_name") <- attr(curr_data, "dataset_name") %||% "Dataset"

  .undo_stack <<- c(list(list(
    data = snapshot,
    description = action_name,
    timestamp = Sys.time()
  )), .undo_stack)

  if (length(.undo_stack) > .max_undo_levels) {
    .undo_stack <<- .undo_stack[1:.max_undo_levels]
  }

  .redo_stack <<- list()
  rrad_write_atomic_checkpoint(curr_data)
}

# Helper to read dataset from path
load_dataset_from_file <- function(file_path) {
  if (file_path == "mtcars") {
    df <- datasets::mtcars
    attr(df, "dataset_name") <- "mtcars"
    return(df)
  }
  if (file_path == "iris") {
    df <- datasets::iris
    attr(df, "dataset_name") <- "iris"
    return(df)
  }
  if (!file.exists(file_path)) {
    stop(paste("File not found:", file_path))
  }

  ext <- tolower(tools::file_ext(file_path))
  df <- switch(ext,
    "sav" = {
      if (requireNamespace("haven", quietly = TRUE)) {
        haven::read_sav(file_path)
      } else {
        stop("haven package required to read .sav files")
      }
    },
    "csv" = {
      read.csv(file_path, stringsAsFactors = FALSE, check.names = FALSE)
    },
    "xlsx" = ,
    "xls" = {
      if (requireNamespace("readxl", quietly = TRUE)) {
        readxl::read_excel(file_path)
      } else {
        stop("readxl package required to read .xlsx/.xls files")
      }
    },
    "rds" = {
      readRDS(file_path)
    },
    "ods" = {
      if (requireNamespace("readODS", quietly = TRUE)) {
        readODS::read_ods(file_path)
      } else {
        stop("readODS package required to read .ods files")
      }
    },
    stop(paste("Unsupported file format:", ext))
  )

  df <- as.data.frame(df)
  attr(df, "dataset_name") <- basename(file_path)
  df
}

# Helper to export dataset to file (CSV, SAV, XLSX, ODS)
export_dataset_to_file <- function(file_path, format = NULL) {
  df <- .GlobalEnv$.active_data
  fmt <- tolower(if (!is.null(format) && nzchar(format)) format else tools::file_ext(file_path))
  switch(fmt,
    "csv" = {
      write.csv(df, file = file_path, row.names = FALSE, na = "")
    },
    "sav" = {
      if (requireNamespace("haven", quietly = TRUE)) {
        haven::write_sav(df, path = file_path)
      } else {
        stop("haven package required to export .sav files")
      }
    },
    "xlsx" = {
      if (requireNamespace("writexl", quietly = TRUE)) {
        writexl::write_xlsx(df, path = file_path)
      } else {
        stop("writexl package required to export .xlsx files")
      }
    },
    "ods" = {
      if (requireNamespace("readODS", quietly = TRUE)) {
        readODS::write_ods(df, path = file_path)
      } else {
        stop("readODS package required to export .ods files")
      }
    },
    stop(paste("Unsupported export format:", fmt))
  )
  list(success = TRUE, file_path = file_path, format = fmt)
}

# Main command dispatcher
handle_request <- function(req) {
  req_id <- if (!is.null(req$request_id)) req$request_id else req$id
  cmd <- req$command_type %||% req$command
  args <- req$args %||% list()
  if (is.null(cmd) && !is.null(req$r_code)) {
    cmd <- "EXECUTE_ANALYSIS"
    args$r_code <- req$r_code
  }

  response <- list(
    request_id = req_id,
    status = "ok"
  )

  switch(cmd,
    "PING" = {
      response$version <- R.version.string
      response$active_dataset <- attr(.GlobalEnv$.active_data, "dataset_name")
    },

    "LOAD_DATASET" = {
      file_path <- args$file_path
      df <- load_dataset_from_file(file_path)
      assign(".active_data", df, envir = .GlobalEnv)
      .undo_stack <<- list()
      .redo_stack <<- list()
      rrad_write_atomic_checkpoint(df)
      response$dataset_name <- attr(df, "dataset_name")
      response$row_count <- nrow(df)
      response$column_count <- ncol(df)
      response$variables <- extract_variable_metadata(df)
    },

    "GET_METADATA" = {
      df <- .GlobalEnv$.active_data
      response$dataset_name <- attr(df, "dataset_name") %||% "Untitled"
      response$row_count <- nrow(df)
      response$column_count <- ncol(df)
      response$variables <- extract_variable_metadata(df)
    },

    "UPDATE_VARIABLE" = {
      col_name <- args$col_name
      updates <- args$updates
      df <- .GlobalEnv$.active_data
      rrad_push_undo_checkpoint(paste("Update variable", col_name))
      df <- mutate_variable_metadata(df, col_name, updates)
      assign(".active_data", df, envir = .GlobalEnv)
      rrad_write_atomic_checkpoint(df)
      response$variables <- extract_variable_metadata(df)
    },

    "GET_DATA_PAGE" = {
      df <- .GlobalEnv$.active_data
      offset <- as.integer(args$offset %||% 0)
      limit <- as.integer(args$limit %||% 100)
      total_rows <- nrow(df)

      if (total_rows == 0 || offset >= total_rows) {
        response$rows <- list()
        response$total_rows <- total_rows
        response$offset <- offset
      } else {
        start_idx <- offset + 1
        end_idx <- min(total_rows, offset + limit)
        sub_df <- df[start_idx:end_idx, , drop = FALSE]

        # Format rows for JSON transmission
        # Convert columns to simple primitives
        clean_df <- as.data.frame(lapply(sub_df, function(x) {
          if (inherits(x, "haven_labelled")) as.character(haven::as_factor(x))
          else if (is.factor(x)) as.character(x)
          else x
        }), check.names = FALSE)

        row_list <- lapply(seq_len(nrow(clean_df)), function(r) {
          as.list(clean_df[r, , drop = FALSE])
        })

        response$rows <- row_list
        response$total_rows <- total_rows
        response$offset <- offset
        response$limit <- limit
      }
    },

    "EXPORT_DATASET" = {
      file_path <- args$file_path
      fmt <- args$format
      res <- export_dataset_to_file(file_path, fmt)
      response$file_path <- res$file_path
      response$format <- res$format
    },

    "RENAME_VARIABLE" = {
      old_name <- args$old_name
      new_name <- trimws(args$new_name)
      df <- .GlobalEnv$.active_data
      idx <- match(old_name, names(df))
      if (!is.na(idx) && nzchar(new_name)) {
        rrad_push_undo_checkpoint(paste("Rename variable", old_name))
        names(df)[idx] <- new_name
        assign(".active_data", df, envir = .GlobalEnv)
        rrad_write_atomic_checkpoint(df)
      }
      response$variables <- extract_variable_metadata(df)
      response$column_count <- ncol(df)
    },

    "ADD_VARIABLE" = {
      col_name <- trimws(args$name %||% "")
      if (!nzchar(col_name)) {
        col_name <- paste0("VAR", sprintf("%05d", ncol(.GlobalEnv$.active_data) + 1))
      }
      measure <- args$measure %||% "scale"
      label <- args$label %||% col_name
      df <- .GlobalEnv$.active_data
      rrad_push_undo_checkpoint("Add variable")
      n_rows <- nrow(df)
      if (n_rows == 0) n_rows <- 1
      df[[col_name]] <- rep(NA_real_, n_rows)
      attr(df[[col_name]], "label") <- label
      assign(".active_data", df, envir = .GlobalEnv)
      rrad_write_atomic_checkpoint(df)
      response$dataset_name <- attr(df, "dataset_name") %||% "Untitled"
      response$row_count <- nrow(df)
      response$column_count <- ncol(df)
      response$variables <- extract_variable_metadata(df)
    },

    "UPDATE_CELL" = {
      row_idx <- as.integer(args$row_index)
      col_name <- as.character(args$col_name)
      val <- args$value
      df <- .GlobalEnv$.active_data

      if (!is.na(row_idx) && row_idx >= 1 && row_idx <= nrow(df) && col_name %in% names(df)) {
        rrad_push_undo_checkpoint(paste("Edit cell (", col_name, ", row ", row_idx, ")", sep = ""))
        if (is.null(val) || (is.character(val) && !nzchar(trimws(val)))) {
          df[row_idx, col_name] <- NA
        } else if (is.numeric(df[[col_name]])) {
          num_val <- suppressWarnings(as.numeric(val))
          df[row_idx, col_name] <- if (is.na(num_val)) NA else num_val
        } else if (is.factor(df[[col_name]])) {
          str_val <- as.character(val)
          if (!(str_val %in% levels(df[[col_name]]))) {
            levels(df[[col_name]]) <- c(levels(df[[col_name]]), str_val)
          }
          df[row_idx, col_name] <- str_val
        } else {
          df[row_idx, col_name] <- as.character(val)
        }
        assign(".active_data", df, envir = .GlobalEnv)
        rrad_write_atomic_checkpoint(df)
      }
      response$dataset_name <- attr(df, "dataset_name") %||% "Untitled"
      response$row_count <- nrow(df)
      response$column_count <- ncol(df)
      response$variables <- extract_variable_metadata(df)
    },

    "INSERT_ROW" = {
      df <- .GlobalEnv$.active_data
      n_rows <- nrow(df)
      pos <- as.integer(args$position %||% (n_rows + 1))
      if (is.na(pos) || pos < 1) pos <- 1
      if (pos > n_rows + 1) pos <- n_rows + 1

      rrad_push_undo_checkpoint("Insert row")

      new_row <- as.data.frame(lapply(df, function(col) {
        if (is.numeric(col)) NA_real_
        else if (is.factor(col)) factor(NA, levels = levels(col))
        else NA_character_
      }), stringsAsFactors = FALSE)
      names(new_row) <- names(df)

      if (n_rows == 0) {
        df <- new_row
      } else if (pos == 1) {
        df <- rbind(new_row, df)
      } else if (pos > n_rows) {
        df <- rbind(df, new_row)
      } else {
        df <- rbind(df[1:(pos - 1), , drop = FALSE], new_row, df[pos:n_rows, , drop = FALSE])
      }
      rownames(df) <- NULL
      assign(".active_data", df, envir = .GlobalEnv)
      rrad_write_atomic_checkpoint(df)
      response$dataset_name <- attr(df, "dataset_name") %||% "Untitled"
      response$row_count <- nrow(df)
      response$column_count <- ncol(df)
      response$variables <- extract_variable_metadata(df)
    },

    "DELETE_ROW" = {
      df <- .GlobalEnv$.active_data
      row_idx <- as.integer(args$row_indices %||% args$row_index)
      row_idx <- row_idx[!is.na(row_idx) & row_idx >= 1 & row_idx <= nrow(df)]
      if (length(row_idx) > 0) {
        rrad_push_undo_checkpoint("Delete row")
        df <- df[-row_idx, , drop = FALSE]
        rownames(df) <- NULL
        assign(".active_data", df, envir = .GlobalEnv)
        rrad_write_atomic_checkpoint(df)
      }
      response$dataset_name <- attr(df, "dataset_name") %||% "Untitled"
      response$row_count <- nrow(df)
      response$column_count <- ncol(df)
      response$variables <- extract_variable_metadata(df)
    },

    "BATCH_UPDATE_CELLS" = {
      start_row <- as.integer(args$start_row %||% 1)
      start_col_name <- as.character(args$start_col %||% "")
      matrix_data <- args$matrix %||% list()
      df <- .GlobalEnv$.active_data

      if (length(matrix_data) > 0) {
        rrad_push_undo_checkpoint(paste("Paste", length(matrix_data), "rows"))
        col_names <- names(df)
        start_col_idx <- match(start_col_name, col_names)
        if (is.na(start_col_idx)) start_col_idx <- 1

        num_paste_rows <- length(matrix_data)
        needed_rows <- start_row + num_paste_rows - 1

        if (needed_rows > nrow(df)) {
          extra_count <- needed_rows - nrow(df)
          extra_df <- as.data.frame(lapply(df, function(col) {
            if (is.numeric(col)) rep(NA_real_, extra_count)
            else if (is.factor(col)) factor(rep(NA, extra_count), levels = levels(col))
            else rep(NA_character_, extra_count)
          }), stringsAsFactors = FALSE)
          names(extra_df) <- names(df)
          df <- rbind(df, extra_df)
          rownames(df) <- NULL
        }

        for (r in seq_len(num_paste_rows)) {
          row_vals <- matrix_data[[r]]
          curr_r <- start_row + r - 1
          if (is.list(row_vals) || is.vector(row_vals)) {
            for (c in seq_along(row_vals)) {
              curr_c <- start_col_idx + c - 1
              if (curr_c <= ncol(df)) {
                c_name <- col_names[curr_c]
                val <- row_vals[[c]]
                if (is.null(val) || (is.character(val) && !nzchar(trimws(val)))) {
                  df[curr_r, c_name] <- NA
                } else if (is.numeric(df[[c_name]])) {
                  num_val <- suppressWarnings(as.numeric(val))
                  df[curr_r, c_name] <- if (is.na(num_val)) NA else num_val
                } else if (is.factor(df[[c_name]])) {
                  str_val <- as.character(val)
                  if (!(str_val %in% levels(df[[c_name]]))) {
                    levels(df[[c_name]]) <- c(levels(df[[c_name]]), str_val)
                  }
                  df[curr_r, c_name] <- str_val
                } else {
                  df[curr_r, c_name] <- as.character(val)
                }
              }
            }
          }
        }
        assign(".active_data", df, envir = .GlobalEnv)
        rrad_write_atomic_checkpoint(df)
      }
      response$dataset_name <- attr(df, "dataset_name") %||% "Untitled"
      response$row_count <- nrow(df)
      response$column_count <- ncol(df)
      response$variables <- extract_variable_metadata(df)
    },

    "FIND_REPLACE" = {
      find_text <- as.character(args$find_text %||% "")
      replace_text <- as.character(args$replace_text %||% "")
      col_name <- args$col_name
      match_entire <- as.logical(args$match_entire_cell %||% TRUE)
      match_case <- as.logical(args$match_case %||% FALSE)
      use_regex <- as.logical(args$use_regex %||% FALSE)

      df <- .GlobalEnv$.active_data
      target_cols <- if (is.null(col_name) || col_name == "all" || !nzchar(col_name)) {
        names(df)
      } else {
        intersect(col_name, names(df))
      }

      total_replaced <- 0

      if (nzchar(find_text) && length(target_cols) > 0) {
        rrad_push_undo_checkpoint(paste("Find & Replace '", find_text, "'", sep = ""))
        pat <- if (use_regex) {
          if (match_entire) paste0("^", find_text, "$") else find_text
        } else {
          if (match_entire) paste0("^\\Q", find_text, "\\E$") else paste0("\\Q", find_text, "\\E")
        }

        for (c_name in target_cols) {
          col_vals <- df[[c_name]]
          col_str <- as.character(col_vals)
          matches <- grepl(pat, col_str, ignore.case = !match_case, perl = TRUE)
          matches[is.na(col_vals)] <- FALSE

          if (any(matches)) {
            n_matches <- sum(matches)
            total_replaced <- total_replaced + n_matches

            new_str <- if (match_entire) {
              rep(replace_text, n_matches)
            } else {
              gsub(pat, replace_text, col_str[matches], ignore.case = !match_case, perl = TRUE)
            }

            if (is.numeric(col_vals)) {
              num_new <- suppressWarnings(as.numeric(new_str))
              df[matches, c_name] <- num_new
            } else if (is.factor(col_vals)) {
              uniq_new <- unique(new_str)
              for (u in uniq_new) {
                if (!(u %in% levels(df[[c_name]]))) {
                  levels(df[[c_name]]) <- c(levels(df[[c_name]]), u)
                }
              }
              df[matches, c_name] <- new_str
            } else {
              df[matches, c_name] <- new_str
            }
          }
        }
        assign(".active_data", df, envir = .GlobalEnv)
        rrad_write_atomic_checkpoint(df)
      }

      response$replaced_count <- total_replaced
      response$dataset_name <- attr(df, "dataset_name") %||% "Untitled"
      response$row_count <- nrow(df)
      response$column_count <- ncol(df)
      response$variables <- extract_variable_metadata(df)
    },

    "COMPUTE_VARIABLE" = {
      target_var <- trimws(args$target_var)
      expr_str <- args$expression
      df <- .GlobalEnv$.active_data
      rrad_reset_blocks()

      # Emit syntax block for transparency
      syntax_text <- paste0(".active_data[['", target_var, "']] <- with(.active_data, ", expr_str, ")")
      rrad_emit_syntax(syntax_text)

      calc_val <- tryCatch({
        val <- with(df, eval(parse(text = expr_str)))
        if (length(val) == 1 && nrow(df) > 1) {
          val <- rep(val, nrow(df))
        }
        val
      }, error = function(e) {
        stop(paste("Formula evaluation error:", conditionMessage(e)))
      })

      rrad_push_undo_checkpoint(paste("Compute variable", target_var))
      df[[target_var]] <- calc_val
      assign(".active_data", df, envir = .GlobalEnv)
      rrad_write_atomic_checkpoint(df)

      # Emit summary table of computed variable
      if (is.numeric(calc_val)) {
        summary_df <- data.frame(
          Target_Variable = target_var,
          Valid_N = sum(!is.na(calc_val)),
          Mean = round(mean(calc_val, na.rm = TRUE), 4),
          Std_Dev = round(sd(calc_val, na.rm = TRUE), 4),
          Min = round(min(calc_val, na.rm = TRUE), 4),
          Max = round(max(calc_val, na.rm = TRUE), 4)
        )
        rrad_emit_table(summary_df, title = paste("Compute Variable:", target_var))
      } else {
        rrad_emit_text(paste("Computed variable", target_var, "with", length(calc_val), "values."), title = "Compute Variable")
      }

      response$dataset_name <- attr(df, "dataset_name") %||% "Untitled"
      response$row_count <- nrow(df)
      response$column_count <- ncol(df)
      response$variables <- extract_variable_metadata(df)
      response$blocks <- rrad_get_blocks()
    },

    "UNDO" = {
      if (length(.undo_stack) > 0) {
        prev_entry <- .undo_stack[[1]]
        .undo_stack <<- .undo_stack[-1]

        # Push current state to redo stack
        .redo_stack <<- c(list(list(
          data = .GlobalEnv$.active_data,
          description = prev_entry$description,
          timestamp = Sys.time()
        )), .redo_stack)

        if (length(.redo_stack) > .max_undo_levels) {
          .redo_stack <<- .redo_stack[1:.max_undo_levels]
        }

        assign(".active_data", prev_entry$data, envir = .GlobalEnv)
        rrad_write_atomic_checkpoint(prev_entry$data)

        response$can_undo <- length(.undo_stack) > 0
        response$can_redo <- length(.redo_stack) > 0
        response$action <- prev_entry$description
        response$dataset_name <- attr(prev_entry$data, "dataset_name") %||% "Untitled"
        response$row_count <- nrow(prev_entry$data)
        response$column_count <- ncol(prev_entry$data)
        response$variables <- extract_variable_metadata(prev_entry$data)
      } else {
        response$status <- "error"
        response$error <- "Nothing to undo"
      }
    },

    "REDO" = {
      if (length(.redo_stack) > 0) {
        next_entry <- .redo_stack[[1]]
        .redo_stack <<- .redo_stack[-1]

        # Push current state to undo stack
        .undo_stack <<- c(list(list(
          data = .GlobalEnv$.active_data,
          description = next_entry$description,
          timestamp = Sys.time()
        )), .undo_stack)

        if (length(.undo_stack) > .max_undo_levels) {
          .undo_stack <<- .undo_stack[1:.max_undo_levels]
        }

        assign(".active_data", next_entry$data, envir = .GlobalEnv)
        rrad_write_atomic_checkpoint(next_entry$data)

        response$can_undo <- length(.undo_stack) > 0
        response$can_redo <- length(.redo_stack) > 0
        response$action <- next_entry$description
        response$dataset_name <- attr(next_entry$data, "dataset_name") %||% "Untitled"
        response$row_count <- nrow(next_entry$data)
        response$column_count <- ncol(next_entry$data)
        response$variables <- extract_variable_metadata(next_entry$data)
      } else {
        response$status <- "error"
        response$error <- "Nothing to redo"
      }
    },

    "GET_UNDO_STATUS" = {
      response$can_undo <- length(.undo_stack) > 0
      response$can_redo <- length(.redo_stack) > 0
      response$undo_action <- if (length(.undo_stack) > 0) .undo_stack[[1]]$description else NULL
      response$redo_action <- if (length(.redo_stack) > 0) .redo_stack[[1]]$description else NULL
    },

    "RESTORE_EMERGENCY_CHECKPOINT" = {
      cfg_dir <- file.path(Sys.getenv("HOME"), ".config", "r-rad-studio", "autosave")
      checkpoint_path <- file.path(cfg_dir, "checkpoint.rds")
      if (!file.exists(checkpoint_path)) {
        checkpoint_path <- file.path(tempdir(), "rrad_checkpoint.rds")
      }

      if (file.exists(checkpoint_path)) {
        rrad_push_undo_checkpoint("Before Emergency Restore")
        df <- readRDS(checkpoint_path)
        assign(".active_data", df, envir = .GlobalEnv)
        response$dataset_name <- attr(df, "dataset_name") %||% "Restored Dataset"
        response$row_count <- nrow(df)
        response$column_count <- ncol(df)
        response$variables <- extract_variable_metadata(df)
        response$can_undo <- length(.undo_stack) > 0
        response$can_redo <- length(.redo_stack) > 0
      } else {
        response$status <- "error"
        response$error <- "No emergency recovery checkpoint found on disk."
      }
    },

    "CHECK_EMERGENCY_CHECKPOINT" = {
      cfg_dir <- file.path(Sys.getenv("HOME"), ".config", "r-rad-studio", "autosave")
      checkpoint_path <- file.path(cfg_dir, "checkpoint.rds")
      if (file.exists(checkpoint_path)) {
        info <- file.info(checkpoint_path)
        response$exists <- TRUE
        response$modified <- as.character(info$mtime)
        response$size_bytes <- info$size
      } else {
        response$exists <- FALSE
      }
    },

    "LOAD_RAW_DATASET" = {
      rows <- args$rows
      vars <- args$variables
      ds_name <- args$dataset_name %||% "Project Dataset"

      if (is.null(rows) || length(rows) == 0) {
        col_names <- vapply(vars, function(v) v$name, character(1))
        df <- as.data.frame(matrix(ncol = length(col_names), nrow = 0))
        names(df) <- col_names
      } else {
        df <- as.data.frame(do.call(rbind, lapply(rows, as.data.frame, stringsAsFactors = FALSE)), stringsAsFactors = FALSE)
      }
      attr(df, "dataset_name") <- ds_name
      assign(".active_data", df, envir = .GlobalEnv)
      .undo_stack <<- list()
      .redo_stack <<- list()
      rrad_write_atomic_checkpoint(df)
      response$dataset_name <- ds_name
      response$row_count <- nrow(df)
      response$column_count <- ncol(df)
      response$variables <- extract_variable_metadata(df)
    },

    "EXECUTE_ANALYSIS" = {
      r_code <- args$r_code %||% req$r_code
      rrad_reset_blocks()

      # Emit syntax block
      rrad_emit_syntax(r_code)

      # Evaluate safely
      eval_res <- tryCatch({
        withCallingHandlers(
          {
            # Evaluate in global environment so .active_data is accessible
            parsed <- parse(text = r_code)
            eval_output <- eval(parsed, envir = .GlobalEnv)

            # If the user script returned a ggplot object or data.frame without calling rrad_emit_*
            if (inherits(eval_output, "ggplot")) {
              rrad_emit_plot(eval_output, title = "Plot Output")
            } else if (inherits(eval_output, "data.frame") && length(rrad_get_blocks()) <= 1) {
              rrad_emit_table(eval_output, title = "Result Table")
            }

            eval_output
          },
          warning = function(w) {
            rrad_emit_warning(conditionMessage(w))
            invokeRestart("muffleWarning")
          },
          message = function(m) {
            msg <- conditionMessage(m)
            # Only emit non-empty informative messages
            if (nzchar(trimws(msg))) {
              rrad_emit_text(trimws(msg), title = "R Message")
            }
          }
        )
      }, error = function(e) {
        rrad_emit_error(conditionMessage(e))
        NULL
      })

      response$blocks <- rrad_get_blocks()
    },

    {
      response$status <- "error"
      response$error <- paste("Unknown command:", cmd)
    }
  )

  response
}

# Helper operator
`%||%` <- function(a, b) if (!is.null(a)) a else b

# Main stdio event loop
main_loop <- function() {
  con <- file("stdin", "r")
  on.exit(close(con))

  # Send ready signal
  ready_msg <- list(type = "READY", version = R.version.string, active_dataset = "mtcars")
  cat(paste0(toJSON(ready_msg, auto_unbox = TRUE), "\n"))
  flush(stdout())

  while (TRUE) {
    line <- readLines(con, n = 1, warn = FALSE)
    if (length(line) == 0) {
      break # End of stream / EOF
    }
    line <- trimws(line)
    if (!nzchar(line)) next

    res <- tryCatch({
      req <- fromJSON(line, simplifyVector = FALSE)
      handle_request(req)
    }, error = function(e) {
      list(status = "error", error = conditionMessage(e))
    })

    # Output single-line JSON followed by newline
    json_out <- toJSON(res, auto_unbox = TRUE, null = "null", digits = 8)
    cat(paste0(json_out, "\n"))
    flush(stdout())
  }
}

# Run when invoked via Rscript
if (!interactive() && is.null(getOption("rrad.no_main_loop"))) {
  main_loop()
}
