# zalgo codec

This crate is a port of the python code by [Scott Conner](https://github.com/DaCoolOne/DumbIdeas/tree/main/reddit_ph_compressor) to Rust. All credit goes to him.

Using the functions defined in this crate you can transform an ASCII string into a unicode string that is a single “character” wide.
This string will almost surely be larger than the original in terms of bytes, but the encoding is reversible.
The crate also provides functions to encode python code and wrap the result in a decoder that decodes and executes 
the encoded string. This way the file looks very different, but executes the same way as before.
This lets you do the mother of all refactoring by converting your entire python program into a single line of code. 
Can not encode carriage returns, so files written on non-unix operating systems might not work.
