# Iron Core :)

Experiments with hosting the .NET Runtime in Rust. I am playing this
game solely on Ubuntu Linux at this time, but some concepts might be helpful elsewhere, too.
This is also mostly old code I recycled, so no guarantees are made for this beast to be readable.

## Caveat emptor

At this point in time, the host executes but calls into the .NET application
result in an unhandled exception:

```
System.IO.FileNotFoundException: Could not load file or assembly 'System.Runtime, Version=7.0.0.0, Culture=neutral, PublicKeyToken=b03f5f7f11d50a3a'. The system cannot find the file specified.

File name: 'System.Runtime, Version=7.0.0.0, Culture=neutral, PublicKeyToken=b03f5f7f11d50a3a'
```

I'm not entirely sure yet how to point the system to the right files. 
