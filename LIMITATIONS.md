# Limitations

`max_rss`

As far as I know, there's no way to accurately measure the `max_rss` of a child process, if the child process uses less memory than the process that created it.

Copying the `max_rss` value from the parent to the child seems to be baked into linux:

https://github.com/torvalds/linux/blob/acc657692aed438e9931438f8c923b2b107aebf9/fs/exec.c#L1033

I'm not the only one who's come across this issue, either:

- https://jkz.wtf/random-linux-oddity-1-ru_maxrss
- https://github.com/ziglang/gotta-go-fast/issues/23
- https://github.com/golang/go/issues/32054
