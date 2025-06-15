## About
I'm really missing sha256sum tool in Windows, so creating it by myself. I would like to ask you forgiveness beforehand for all this shitty code, as I'm not a real programmer :-D
Hope this command line tool will be useful for you. You can submit issues here on GitHub. I promise I will check them from time to time and fix.
This tool is not scanning directory recursively! So if you want to calculate sha256 for all subfolders, you need to launch a tool for every subfolder.

This tool is not saving any data yet! If you want to save output, use sha256sum.exe > filename.sha256.

Also, it uses stderr output to report errors. And it uses different exit codes for different issues. You can find them [here](https://github.com/helen-beauty/sha256sum/blob/main/exitcodes.txt)


## Download
Download [stable version](https://github.com/helen-beauty/sha256sum/blob/main/sha256sum.exe)

## How to use
Usage is primitive

**Usage:**

_sha256sum.exe {key} {directory or filename}_

Keys: 
- -c - calculates sha256sum for file or folder specified
- -v - verify sha256sum for .sha256 file.

_**Remember to use quotes "" for file names or paths with spaces in their name.**_
