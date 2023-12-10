#! /usr/bin/env python

import gaps_online.tof.converters
import argparse 
import numpy as np

if __name__ == '__main__':

    parser = argparse.ArgumentParser(description='Convert NTS environmental data to hdf file')
    parser.add_argument('infile',
                        default="",
                        type=str,
                        help="Input (stream) file")
    parser.add_argument('--run-id',
                        default=0,
                        type=int,
                        help="RunID for the moni file name")
    data = converters.extract_moni_data(args.infile)
    data = np.array(data)
    outname = f'nts_moni_{args.run_id}.h5'
    converters.save_to_hdf(data, outname)
