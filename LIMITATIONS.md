# Limitations

`max_rss`

As far as I know, there's no way to accurately measure the `max_rss` of a child process, if the child process uses less memory than the process that created it.

Copying the `max_rss` value from the parent to the child seems to be baked into linux:

https://github.com/torvalds/linux/blob/acc657692aed438e9931438f8c923b2b107aebf9/fs/exec.c#L1033

I'm not the only one who's come across this issue, either:

- https://jkz.wtf/random-linux-oddity-1-ru_maxrss
- https://tbrindus.ca/sometimes-the-kernel-lies-about-process-memory-usage/
- https://github.com/ziglang/gotta-go-fast/issues/23
- https://github.com/golang/go/issues/32054

If you read `man 5 proc` you'll see that it mentions the `rss` field and some others are inaccurate, and then it recommends reading `/proc/$PID/smaps` instead.

So, I think the best workaround for users desiring to capture an accurate `rss` value, is to use `gdb` and then read `/proc/smaps_rollup`. For example:

1. `gdb --args ./my_program some arguments`
2. `(gdb) catch syscall exit exit_group` (catch the exit syscall)
3. `(gdb) condition 1 $_thread == 1` (optional, but if the program is multithreaded this should help you catch the final exit in most cases. Obviously this will change depending on the debugged program)
4. `(gdb) run`
5. Once the program has been halted before the exit syscall, now dump `/proc/$PID/smaps_rollup` and extract the `Rss: ...` value from there

