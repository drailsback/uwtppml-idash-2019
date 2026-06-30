import sys

def io_gen_list(filename):

	infile = open(filename, 'r')

	dataset = []

	for line in infile:
		dataset.append( list(map(float, line.split(','))) )

	infile.close()

	return dataset

def activate(z):

	if z < -0.5:
		return 0

	if z < 0.5:
		return z + 0.5

	return 1

def classify(x):

	if x >= 0.5:

		return 1

	return 0

if __name__=='__main__':

	fold = sys.argv[1]

	weights_path       = 'C:/Users/davis/projects/idash2019_rust/weights/BC_TCGA_5_folds/fold_{}_weights.csv'
	x_test_matrix_path = 'C:/Users/davis/projects/idash2019_rust/test_vectors/BC_TCGA_5_folds/{}X_test_.csv'
	y_test_matrix_path = 'C:/Users/davis/projects/idash2019_rust/test_vectors/BC_TCGA_5_folds/{}y_test_.csv'

	# weights_path       = 'C:/Users/davis/projects/idash2019_rust/weights/GSE2034_5_folds/fold_{}_weights.csv'
	# x_test_matrix_path = 'C:/Users/davis/projects/idash2019_rust/test_vectors/GSE2034_5_folds/{}X_test_.csv'
	# y_test_matrix_path = 'C:/Users/davis/projects/idash2019_rust/test_vectors/GSE2034_5_folds/{}y_test_.csv'

	weights       = io_gen_list( weights_path.format(fold) )
	x_test_matrix = io_gen_list( x_test_matrix_path.format(fold) )
	y_test_matrix = io_gen_list( y_test_matrix_path.format(fold) )

	test_len = len(y_test_matrix)
	attr_cnt = len(weights)

	results = [0] * test_len

	for i in range(test_len):

		z = sum( [ x_test_matrix[i][x] * weights[x][0] for x in range(attr_cnt) ] )
		o = activate(z)

		results[i] = o

	res_classified = [ classify(results[i]) for i in range(test_len) ]
	y_actual       = [ int(y_test_matrix[i][0]) for i in range(test_len) ]

	correct_classifications =sum( \
		[ 1 if y_actual[i] == results[i] else 0 for i in range(test_len) ])

	accuracy = (1.0 * correct_classifications) / test_len

	print('Correct predictions: {}, Total test cases: {}, Accuracy: {}' \
		.format(correct_classifications, test_len, accuracy))
