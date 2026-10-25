# r-runtime/formatters.R
# Formats R objects, tables, and graphics into structured output blocks

# Execution block accumulator
.rrad_blocks <- list()

rrad_reset_blocks <- function() {
  .rrad_blocks <<- list()
}

rrad_get_blocks <- function() {
  .rrad_blocks
}

rrad_emit_block <- function(block) {
  .rrad_blocks[[length(.rrad_blocks) + 1]] <<- block
}

rrad_emit_syntax <- function(syntax_code) {
  rrad_emit_block(list(
    type = "syntax",
    content = as.character(syntax_code)
  ))
}

rrad_emit_warning <- function(msg) {
  rrad_emit_block(list(
    type = "warning",
    content = as.character(msg)
  ))
}

rrad_emit_error <- function(msg) {
  rrad_emit_block(list(
    type = "error",
    content = as.character(msg)
  ))
}

rrad_emit_text <- function(text, title = NULL) {
  rrad_emit_block(list(
    type = "text",
    title = if (!is.null(title)) as.character(title) else "",
    content = as.character(text)
  ))
}

# Converts a data.frame or table-like object into clean academic HTML
df_to_academic_html <- function(df, title = NULL, subtitle = NULL) {
  df <- as.data.frame(df)
  title_html <- if (!is.null(title) && nzchar(title)) {
    sprintf("<div class=\"rrad-table-title\">%s</div>", htmlEscape(as.character(title)))
  } else ""
  
  subtitle_html <- if (!is.null(subtitle) && nzchar(subtitle)) {
    sprintf("<div class=\"rrad-table-subtitle\">%s</div>", htmlEscape(as.character(subtitle)))
  } else ""

  col_names <- names(df)
  header_cells <- paste0(vapply(col_names, function(cn) {
    sprintf("<th scope=\"col\">%s</th>", htmlEscape(cn))
  }, FUN.VALUE = character(1)), collapse = "")

  # Format rows
  rows_html <- ""
  if (nrow(df) > 0) {
    row_strings <- character(nrow(df))
    for (r in seq_len(nrow(df))) {
      cells <- vapply(seq_along(col_names), function(c) {
        val <- df[r, c]
        if (is.null(val) || is.na(val)) {
          return("<td class=\"text-center text-muted\">.</td>")
        }
        if (is.numeric(val)) {
          # Format float nicely
          formatted <- if (abs(val) < 0.001 && val != 0) {
            sprintf("%.2e", val)
          } else {
            sprintf("%.3f", val)
          }
          # Strip trailing zeros for integers
          if (round(val) == val) {
            formatted <- sprintf("%d", as.integer(val))
          }
          return(sprintf("<td class=\"text-right num-cell\">%s</td>", htmlEscape(formatted)))
        }
        sprintf("<td>%s</td>", htmlEscape(as.character(val)))
      }, FUN.VALUE = character(1))
      row_strings[r] <- sprintf("<tr>%s</tr>", paste0(cells, collapse = ""))
    }
    rows_html <- paste0(row_strings, collapse = "\n")
  }

  sprintf(
    "<div class=\"rrad-table-container my-3\">%s%s<table class=\"rrad-table\">\n<thead><tr>%s</tr></thead>\n<tbody>\n%s\n</tbody>\n</table></div>",
    title_html,
    subtitle_html,
    header_cells,
    rows_html
  )
}

# Basic HTML escaping
htmlEscape <- function(text) {
  if (is.null(text)) return("")
  text <- gsub("&", "&amp;", text, fixed = TRUE)
  text <- gsub("<", "&lt;", text, fixed = TRUE)
  text <- gsub(">", "&gt;", text, fixed = TRUE)
  text <- gsub("\"", "&quot;", text, fixed = TRUE)
  text
}

rrad_emit_table <- function(obj, title = NULL, subtitle = NULL) {
  # If it's a broom tidyable object or model
  if (!is.data.frame(obj)) {
    if (requireNamespace("broom", quietly = TRUE)) {
      try_tidy <- tryCatch(broom::tidy(obj), error = function(e) NULL)
      if (!is.null(try_tidy)) {
        obj <- try_tidy
      } else {
        obj <- as.data.frame(obj)
      }
    } else {
      obj <- as.data.frame(obj)
    }
  }

  # If gt is available and obj is a data frame, gt can also be used
  html_str <- ""
  if (requireNamespace("gt", quietly = TRUE) && inherits(obj, "data.frame")) {
    gt_obj <- tryCatch({
      g <- gt::gt(obj)
      if (!is.null(title) && nzchar(title)) {
        g <- gt::tab_header(g, title = title, subtitle = subtitle)
      }
      gt::as_raw_html(g)
    }, error = function(e) NULL)
    if (!is.null(gt_obj)) {
      html_str <- gt_obj
    }
  }

  # Fallback to academic html if gt failed or not used
  if (!nzchar(html_str)) {
    html_str <- df_to_academic_html(obj, title = title, subtitle = subtitle)
  }

  rrad_emit_block(list(
    type = "html_table",
    title = if (!is.null(title)) as.character(title) else "Analysis Output",
    html = html_str
  ))
}

rrad_emit_plot <- function(plot_obj = NULL, title = NULL, width = 7, height = 4.5) {
  svg_content <- ""
  tmp_file <- tempfile(fileext = ".svg")
  
  tryCatch({
    if (requireNamespace("svglite", quietly = TRUE)) {
      svglite::svglite(tmp_file, width = width, height = height)
    } else {
      svg(tmp_file, width = width, height = height)
    }

    if (!is.null(plot_obj)) {
      print(plot_obj)
    }
    dev.off()

    if (file.exists(tmp_file)) {
      svg_content <- paste(readLines(tmp_file, warn = FALSE), collapse = "\n")
      unlink(tmp_file)
    }
  }, error = function(e) {
    if (dev.cur() > 1) dev.off()
    rrad_emit_error(paste("Plot generation error:", e$message))
  })

  if (nzchar(svg_content)) {
    rrad_emit_block(list(
      type = "svg_plot",
      title = if (!is.null(title)) as.character(title) else "Chart",
      svg = svg_content
    ))
  }
}
