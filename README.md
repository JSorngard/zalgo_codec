# zalgo codec

This crate is a port of the [Python code](https://github.com/DaCoolOne/DumbIdeas/tree/main/reddit_ph_compressor) by Scott Conner to Rust. All credit goes to him.

Using the functions defined in this crate you can transform an ASCII string into a unicode string that is a single “character” wide. The encoding is reversible, but the string will be larger than the original in terms of bytes.

The crate also provides functions to encode python code and wrap the result in a decoder that decodes and executes 
the encoded string. This way the file looks very different, but executes the same way as before.
This lets you do the mother of all refactoring by converting your entire python program into a single line of code. 
Can not encode carriage returns, so files written on non-unix operating systems might not work. The file encoding functions will attempt to encode files anyway by ignoring carriage returns.  

# Example
The cursed character at the bottom of this text is the standard "Lorem ipsum" encoded with this crate.
\
\
\
\
\
\
\
\
\
\
\
\
\
E̬͏͍͉͓͕͍͒̀͐̀̈́ͅ͏͌͏͓͉͔͍͔͒̀̀́̌̀̓ͅ͏͎͓͔͔͕͉͉͓͉͎͇͉͔͓̓͒̀́̈́͐̓̀͌̌̀̈́̀̈́ͅͅͅͅ͏͉͕͓͍̀ͅ͏͔͍̈́̀͐ͅ͏͉͎͉͉͕͎͔͕͔͒̀̓̈́̈́̀̀͌́͂͏͔͒̀̀̈́ͅͅ͏͌͏͍͇͎͉͒̀́́̀́͌ͅ