"""
Convert a video to a txt in a format, which i need
Using: https://github.com/ivanl-exe/image-to-ascii/
"""

import cv2
import os
import shutil

OUT_DIR = 'assets/output'
GEN_TXT = 'assets/gen.txt'
CONVERT = False

video = cv2.VideoCapture(r'/home/marc/5fps.mp4')

# write all frames to an output dir
current_frame = 0
if CONVERT:
    # make sure to delete leftover output
    shutil.rmtree(OUT_DIR)
    os.makedirs(OUT_DIR)
    while True:
        ret, frame = video.read()
        if ret:
            name = OUT_DIR + '/frame' + str(current_frame) + '.jpg'
            cv2.imwrite(name, frame)
            print(f"Frame to jpg: {current_frame}")
            current_frame += 1
        else:
            break

# convert to ascii
shutil.rmtree(GEN_TXT)
for f in os.listdir(OUT_DIR):
    filename = os.path.abspath(OUT_DIR + '/' + f)
    print(f'Processing: {filename}')
    os.system(f'../image-to-ascii/target/release/image_to_ascii 21 10 true {filename} >> {GEN_TXT}')

# cleanup
video.release()
cv2.destroyAllWindows()
