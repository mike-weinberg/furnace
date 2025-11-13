# Principles of Tidy Data

## Introduction

Data organization significantly impacts the efficiency of data analysis workflows. Most datasets do not originate in a structured format suitable for analysis. Instead, data is frequently organized to facilitate data entry or other operational objectives rather than analytical processing. This document outlines the principles of tidy data as established by Hadley Wickham, providing a standardized framework for data organization.

## The Three Rules of Tidy Data

Tidy data adheres to three interrelated principles that define its structure:

1. **Each variable is a column; each column is a variable.** Variables represent distinct attributes or measurements being tracked in the dataset.

2. **Each observation is a row; each row is an observation.** Rows represent individual cases or instances where measurements were taken.

3. **Each value is a cell; each cell is a single value.** Data points are stored as singular values rather than in compound formats.

These three rules create a standardized structure that enables consistent application of analytical tools and transformations.

## Advantages of Tidy Data

The tidy data framework provides two primary advantages:

**Consistency**: A uniform data structure simplifies the application of analytical tools. When datasets follow the same organizational principles, tools designed for that structure can be applied consistently across different datasets without requiring custom adaptations for each data source.

**Computational efficiency**: Storing variables in columns aligns with the vectorized operations common in statistical programming environments. Most built-in functions in languages like R operate on vectors of values, making computations on tidy data more efficient and intuitive.

## Common Problems with Messy Data

Data fails to meet tidy standards for several identifiable reasons:

1. **Organizational priorities**: Data is often structured to optimize data entry, storage, or display rather than analysis. Forms, spreadsheets, and databases frequently organize data in ways that serve operational needs but complicate analytical workflows.

2. **Lack of familiarity**: Many data creators have limited exposure to tidy data principles and may not have experience working with data in analytical contexts.

These factors result in datasets that require substantial preprocessing before analysis can proceed.

## Types of Messy Data

### Type 1: Column Headers Contain Data Values

In this pattern, column names encode data values rather than variable names.

**Example**: Billboard Music Rankings

In the untidy format, the dataset contains columns labeled `wk1`, `wk2`, `wk3`, through `wk76`, where each column represents a different week. The column name itself contains the week number, which is a data value rather than a variable name.

**Structure**:
- Original dimensions: 317 rows × 79 columns
- Problem: Week numbers stored as column names instead of as a variable

**Solution**: Transform wide format to long format by:
- Creating a new `week` column to store the week numbers
- Creating a new `rank` column to store the ranking values
- Result: 5,307 rows × 5 columns

This transformation converts encoded information in column names into explicit data values in rows.

### Type 2: Multiple Variables Stored in Column Names

Column names may contain multiple pieces of information that should be separated into distinct variables.

**Example**: WHO Tuberculosis Dataset

Column names like `sp_m_014` encode three separate variables:
- Diagnosis method: `sp` (smear positive), `rel` (relapse), `ep` (extrapulmonary)
- Gender: `m` (male), `f` (female)
- Age range: `014` (0-14 years), `1524` (15-24 years), `2534` (25-34 years)

**Solution**: Split compound column names into multiple variables by:
- Parsing the column name using a delimiter (underscore in this case)
- Creating separate columns for diagnosis method, gender, and age range
- Storing the count values in a separate column

### Type 3: Variables Stored in Both Rows and Columns

Some datasets mix the storage of variables between row headers and column headers.

**Example**: Virginia Death Rates

In the untidy format, age groups appear as row headers and years appear as column headers, with death rates as cell values. This structure makes it impossible to use the data with standard analytical functions because variables are not properly isolated in columns.

**Solution**: Restructure so that:
- `age_group` becomes a column variable
- `year` becomes a column variable
- `death_rate` becomes a column variable
- Each row represents one age group in one year

### Type 4: Multiple Types of Observational Units in One Table

When different types of observations are combined in a single table, the dataset violates the principle that each type of observational unit should form a table.

**Example**: Billboard Dataset (Extended)

The original Billboard dataset combines two types of observations:
- Song characteristics (artist, track name, date entered)
- Weekly rankings (rank in each week)

These represent different observational units and would ideally be separated into two tables with a linking key.

### Type 5: Single Observation Stored Across Multiple Rows

Some datasets spread one logical observation across multiple rows, with each row containing a different measurement or attribute.

**Example**: CMS Patient Experience Dataset

Organizations appear across six rows, with each row representing a different measurement type rather than a complete observation.

**Structure**:
- Organization identifier repeated across multiple rows
- Measurement type stored as a data value in one column
- Measurement value stored in another column

**Solution**: Transform from long to wide format by:
- Using measurement types as new column names
- Consolidating related rows into single observations
- Result: Fewer rows with more columns, where each row represents one organization

### Type 6: Multiple Values in Single Cells

Data may be stored with multiple values combined in a single cell, violating the principle that each cell contains a single value.

**Example**: Tuberculosis Rate Data

The `rate` column stores values as character strings like `"745/19987071"`, combining case counts and population in a single cell.

**Solution**: Split the combined value into separate columns:
- `cases`: Numerator (745)
- `population`: Denominator (19987071)
- Optionally calculate `rate` as cases/population

## Transformation Operations

### Lengthening (Pivot Longer)

Lengthening transforms wide-format data to long-format by converting columns into rows. This operation increases row count while decreasing column count.

**Key parameters**:
- **Columns to pivot**: Specify which columns should be converted to rows
- **Names destination**: Column name for storing the original column names
- **Values destination**: Column name for storing the cell values
- **Drop missing values**: Option to remove artificially created missing values

**Use cases**:
- Column headers contain data values
- Multiple related measurements stored as separate columns

### Widening (Pivot Wider)

Widening transforms long-format data to wide-format by converting rows into columns. This operation decreases row count while increasing column count.

**Key parameters**:
- **Names source**: Column containing values that will become new column names
- **Values source**: Column containing values that will populate the cells
- **Identifier columns**: Columns that uniquely identify each row in the output

**Use cases**:
- One observation spread across multiple rows
- Key-value pair structures that should be columns

### Separating

Separating splits compound values in a single column into multiple columns.

**Key parameters**:
- **Source column**: Column containing compound values
- **Separator**: Character or pattern used to split values
- **Destination columns**: Names of new columns to create

**Use cases**:
- Multiple pieces of information concatenated in one field
- Composite identifiers that should be decomposed

### Uniting

Uniting combines multiple columns into a single column, the inverse of separating.

**Use cases**:
- Creating composite keys from multiple fields
- Concatenating related text fields

## Case Study: Billboard Rankings

This comprehensive example demonstrates multiple tidy data principles:

**Original structure**: 317 rows × 79 columns
- Columns: `artist`, `track`, `date.entered`, `wk1`, `wk2`, ..., `wk76`
- Week numbers encoded in column names
- Missing values where songs dropped off charts

**Transformation steps**:

1. **Lengthen week columns**:
   - Convert `wk1` through `wk76` into two columns: `week` and `rank`
   - Result: 5,307 rows × 5 columns

2. **Clean week values**:
   - Convert `"wk1"` format to numeric value `1`

3. **Remove artificial missing values**:
   - Drop rows where `rank` is missing (songs not on chart)

**Final structure**:
- Each row represents one song's ranking in one week
- Variables properly isolated in columns
- Suitable for time series analysis and visualization

## Case Study: WHO Tuberculosis Data

This example demonstrates separating multiple variables encoded in column names:

**Original structure**:
- Columns like `sp_m_014` encoding diagnosis, gender, and age
- Multiple combination columns creating a wide dataset

**Transformation steps**:

1. **Lengthen measurement columns**:
   - Convert all diagnosis-gender-age combinations into rows
   - Create a column for the compound name and a column for the count

2. **Separate compound names**:
   - Split using underscore delimiter
   - Create `diagnosis`, `gender`, and `age` columns

3. **Type conversion**:
   - Convert character values to categorical factors as appropriate
   - Convert counts to numeric type

**Final structure**:
- Each row represents one diagnosis-gender-age group in one country-year
- All variables explicitly represented as columns
- Suitable for epidemiological analysis

## Case Study: Household Demographics

This example demonstrates handling mixed variable names and values in column headers:

**Original structure**:
- Columns like `dob_child1`, `dob_child2`, `name_child1`, `name_child2`
- Column names contain both variable names (`dob`, `name`) and values (`child1`, `child2`)

**Transformation steps**:

1. **Lengthen with variable name preservation**:
   - Use special `.value` sentinel to indicate which part becomes a variable
   - Split column names into variable name and child identifier

2. **Restructure**:
   - Create `child` column with values 1, 2, etc.
   - Create separate `dob` and `name` columns

**Final structure**:
- Each row represents one child in one household
- Variables clearly separated from observational identifiers

## Practical Considerations

### Pragmatic Flexibility

The definition of what constitutes a "variable" depends on the analytical context. The same dataset may be organized differently for different analyses. It is acceptable and often necessary to transform data multiple times during analysis, restructuring it to suit different analytical or visualization needs.

### Handling Duplicates

When transforming data from long to wide format, duplicate values in the grouping variables create complications. If multiple input rows correspond to one output cell, special handling is required, such as aggregation or list-column creation.

### Computational Cost

Transformations between wide and long formats have computational costs. For very large datasets, careful consideration of data structure at the outset can reduce the need for repeated transformations.

### Data Storage vs. Analysis Format

The tidy data principles apply primarily to data in-memory during analysis. Database storage and file formats may use different organizational schemes for efficiency. The key is to transform data into tidy format when loading it into the analytical environment.

## The Tidyverse Ecosystem

The tidy data framework forms the foundation for an integrated set of tools designed to work with consistently structured data:

- **Data manipulation**: Operations for filtering, selecting, mutating, and summarizing
- **Data transformation**: Tools for reshaping between wide and long formats
- **Visualization**: Graphical systems that expect tidy inputs
- **Modeling**: Statistical functions designed for tidy data structures

This ecosystem demonstrates the practical benefits of data standardization: once data is tidy, a consistent set of tools can be applied regardless of the original data source or domain.

## Summary

Tidy data principles provide a standardized framework for organizing data that:

1. Facilitates consistent application of analytical tools
2. Aligns with computational patterns in statistical software
3. Reduces the cognitive load of working with diverse datasets
4. Enables composition of operations through consistent interfaces

Most real-world data requires transformation to meet tidy standards. Understanding common patterns of messiness enables systematic application of appropriate transformations. While the principles are straightforward, applying them effectively requires practice and judgment about what organizational structure best serves the analytical objectives.

The three core rules—variables in columns, observations in rows, values in cells—provide clear criteria for assessing and improving data organization. Adherence to these principles represents an investment that yields returns through more efficient and reliable analytical workflows.
