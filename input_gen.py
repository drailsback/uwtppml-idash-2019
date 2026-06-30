import os 
from random import randint
from math import floor

# load real-values from csv into 2d list
def get_dataset(path, filename):

	infile = open(os.path.join(path, filename), 'r')

	dataset = []

	for line in infile:
		dataset.append( list(map(float, line.split(','))) )

	infile.close()

	return dataset

# map r.v. dataset to integer and secret share 
def generate_shares(path, filename, ringsize, decimalAcc, dataset):
	
	outfile0 = open(os.path.join(path, filename[:-4] + 'share0.csv'), 'w')
	outfile1 = open(os.path.join(path, filename[:-4] + 'share1.csv'), 'w')

	ringModulus = 2 ** ringsize
	shift = 2 ** decimalAcc

	for line in dataset:

		val = line[0]
		if val < 0:
			val = ringModulus - floor((-1 * val) * shift)
		else:
			val = floor(val * shift)
		
		z0 = randint(0, ringModulus - 1)
		z1 = (int(val) - z0) % ringModulus

		outfile0.write(str(z0))
		outfile1.write(str(z1))

		for i in range(1, len(line)):

			val = line[i]
			if val < 0:
				val = ringModulus - floor((-1 * val) * shift)
			else:
				val = floor(val * shift)

			z0 = randint(0, ringModulus - 1)
			z1 = (int(val) - z0) % ringModulus

			outfile0.write(',' + str(z0))
			outfile1.write(',' + str(z1))

		outfile0.write('\n')
		outfile1.write('\n')
	
			
	outfile0.close()
	outfile1.close()		

# main: generate secret shares for all .csv's in src directory
if __name__ == '__main__':

	ringsize = 64
	decimalAcc = 10

	in_path = 'C:/msys32/home/davis/idash2019/5_fold_data/BC_TCGA_5_folds/' 
	# # out_path = 'C:/msys32/home/davis/idash2019/InputGenerationScripts/LR_Inputs/BC_TCGA_5_folds/'
	out_path = 'C:/Users/davis/projects/idash2019_rust/inputs/BC_TCGA_5_folds'


# C:/Users/davis/projects/idash2019_rust/inputs

	# in_path = 'C:/msys32/home/davis/idash2019/5_fold_data/GSE2034_5_folds/' 
	# out_path = 'C:/Users/davis/projects/idash2019_rust/inputs/GSE2034_5_folds/'


	for filename in os.listdir(in_path):
		if filename.endswith('.csv'):
			dataset = get_dataset(in_path, filename)
			generate_shares(out_path, filename, ringsize, decimalAcc, dataset)


