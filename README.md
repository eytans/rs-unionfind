# Unionfind

A very simple implementation of unionfind in rust.
Uses indexmap as the backbone to support any data structure.
To have a nice API it uses RefCell, so find is still efficient, but not concurrent.