Duckvert is a local file/folder converter. Give it files or folders, and tell it what to convert the contents into.
You can enter as many paths as you want, one per line. A blank line finishes the list. Folders are walked fully.

All converted files go into one output folder (You have to specify). 
Enter the folder when asked, or press Enter to use the first path's directory.

Converted files are named NAME_converted.FORMAT. If two files end up with the same name, the later ones get -2, -3, etc. so nothing is overwritten.

Supported formats:
- Image: jpg, jpeg, png, bmp, tiff, tif, gif, webp, ico
- Video: mp4, mkv, mov, avi, ogg, webm, flv, wmv, m4v, mpeg, mpg, ts, 3gp
- Audio: mp3, wav, flac, aac, m4a, ogg, opus, aiff, wma, amr
- Document: txt, pdf, md, doc, docx, rtf, odt, xls, xlsx, ppt, pptx

Audio/video conversions require ffmpeg to be installed.
