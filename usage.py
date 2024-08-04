# Import the matrix_market module
import rsmtxmkt

# Define the path to the Matrix Market file
filename = "E-MTAB-8362.aggregated_filtered_counts.mtx"

try:
    # Call the Rust function to load the matrix
    matrix = rsmtxmkt.py_load_matrix_market(filename)

    # Access the matrix components from the dictionary
    indptr = matrix["indptr"]
    indices = matrix["indices"]
    values = matrix["values"]

#     # If you want to combine them into triplets for easier processing
#     print("\nMatrix Triplets (row, col, value):")
#     for row_start, row_end in zip(indptr[:-1], indptr[1:]):
#         for idx in range(row_start, row_end):
#             row = indptr.index(row_start)
#             col = indices[idx]
#             value = values[idx]
#             print(f"({row}, {col}): {value}")

except Exception as e:
    print(f"An error occurred: {e}")
