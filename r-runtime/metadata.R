# r-runtime/metadata.R
# Extracts & mutates dataset and variable metadata (SPSS Variable View alignment)

extract_measure_type <- function(col) {
  # Explicitly set attribute takes precedence
  custom_measure <- attr(col, "measure", exact = TRUE)
  if (!is.null(custom_measure) && nzchar(custom_measure)) {
    return(tolower(as.character(custom_measure)))
  }

  # Ordered factor -> ordinal
  if (is.ordered(col)) {
    return("ordinal")
  }

  # Standard factor, character, logical -> nominal
  if (is.factor(col) || is.character(col) || is.logical(col)) {
    return("nominal")
  }

  # Haven labelled types
  if (inherits(col, "haven_labelled")) {
    labels <- attr(col, "labels", exact = TRUE)
    if (!is.null(labels) && length(labels) > 0) {
      return("nominal")
    }
  }

  # Numeric or integer
  if (is.numeric(col)) {
    return("scale")
  }

  # Fallback
  return("nominal")
}

extract_variable_metadata <- function(df) {
  if (is.null(df) || !is.data.frame(df)) {
    return(list())
  }

  col_names <- names(df)
  vars <- list()

  for (i in seq_along(col_names)) {
    col_name <- col_names[i]
    col <- df[[col_name]]

    # Storage type
    storage_type <- typeof(col)
    spss_type <- "Numeric"
    if (is.character(col) || is.factor(col)) {
      spss_type <- "String"
    } else if (inherits(col, "Date") || inherits(col, "POSIXt")) {
      spss_type <- "Date"
    }

    # Width & Decimals
    decimals <- 0
    width <- 8
    if (is.numeric(col) && !is.integer(col)) {
      decimals <- 2
    }

    # Variable Label
    var_label <- attr(col, "label", exact = TRUE)
    if (is.null(var_label) || is.na(var_label)) {
      var_label <- ""
    } else {
      var_label <- as.character(var_label)
    }

    # Value Labels (Values)
    val_labels_list <- list()
    raw_labels <- attr(col, "labels", exact = TRUE)
    if (!is.null(raw_labels) && length(raw_labels) > 0) {
      lbl_names <- names(raw_labels)
      for (k in seq_along(raw_labels)) {
        val_labels_list[[k]] <- list(
          value = as.character(raw_labels[k]),
          label = ifelse(is.null(lbl_names), as.character(raw_labels[k]), lbl_names[k])
        )
      }
    } else if (is.factor(col)) {
      lvls <- levels(col)
      for (k in seq_along(lvls)) {
        val_labels_list[[k]] <- list(
          value = as.character(k),
          label = lvls[k]
        )
      }
    }

    # Measurement level
    measure <- extract_measure_type(col)

    # Alignment
    align <- ifelse(is.numeric(col), "right", "left")

    vars[[i]] <- list(
      name = col_name,
      type = spss_type,
      width = width,
      decimals = decimals,
      label = var_label,
      values = val_labels_list,
      missing = "None",
      columns = 8,
      align = align,
      measure = measure,
      storage_type = storage_type
    )
  }

  vars
}

mutate_variable_metadata <- function(df, col_name, updates) {
  if (is.null(df) || !(col_name %in% names(df))) {
    return(df)
  }

  col <- df[[col_name]]

  if (!is.null(updates$label)) {
    attr(col, "label") <- as.character(updates$label)
  }

  if (!is.null(updates$measure)) {
    attr(col, "measure") <- tolower(as.character(updates$measure))
  }

  if (!is.null(updates$name) && updates$name != col_name && nzchar(updates$name)) {
    new_name <- updates$name
    names(df)[names(df) == col_name] <- new_name
    col_name <- new_name
  }

  df[[col_name]] <- col
  df
}
