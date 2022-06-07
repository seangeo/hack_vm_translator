# Python Script for interop with Coursera grader.
#
import subprocess
import sys

subprocess.call(['./vm', sys.argv[1]])
