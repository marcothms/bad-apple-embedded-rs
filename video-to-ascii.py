"""
Convert a video to a textfile in a format, which i need:

ASCII_TXT:
---
IMAGE
IMAGE
IMAGE
...
---

Using: https://github.com/ivanl-exe/image-to-ascii/
"""


import cv2
import os
import shutil
import argparse


OUTPUT = 'assets/output'
ASCII_TXT = 'assets/ascii.txt'
TOOL_PATH = '../image-to-ascii/target/release/image_to_ascii'


def parse_args():
    """
    Parse args and return them
    """
    parser = argparse.ArgumentParser(description='Convert a video to a contiguous ascii text file.')
    parser.add_argument('--video', help='Input Video')
    parser.add_argument('--width', default='21', help='Width of a single ascii image')
    parser.add_argument('--height', default='10', help='Height of a single ascii image')

    return parser.parse_args()


def main():
    """
    Convert frames to jpgs, convert each to ascii, and append it to the ascii.txt
    """
    args = parse_args()
    current_frame = 0

    if args.video is None:
        print("Must provide a path to a video!")
        exit(1)
    video = cv2.VideoCapture(args.video)

    # make sure to delete leftover output
    try:
        shutil.rmtree(OUTPUT)
    except FileNotFoundError:
        pass
    os.makedirs(OUTPUT)
    while True:
        ret, frame = video.read()
        if ret:
            name = OUTPUT + '/frame' + str(current_frame) + '.jpg'
            cv2.imwrite(name, frame)
            print(f"Frame to jpg: {current_frame}")
            current_frame += 1
        else:
            break

    # convert to ascii
    try:
        os.remove(ASCII_TXT)
    except FileNotFoundError:
        pass

    for f in os.listdir(OUTPUT):
        filename = os.path.abspath(OUTPUT + '/' + f)
        print(f'Processing: {filename}')
        os.system(f'{TOOL_PATH} {args.width} {args.height} true {filename} >> {ASCII_TXT}')

    # cleanup
    video.release()
    cv2.destroyAllWindows()


def check_deps():
    """
    Checks if the deps, used in this script, are present
    """
    if not os.path.isfile(TOOL_PATH):
        print("Missing image-to-ascii at expected location!")
        exit(1)

if __name__ == '__main__':
    check_deps()
    main()
