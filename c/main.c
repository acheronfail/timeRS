/**
 * This is just a proof of concept/scratch file used to prototype C APIs without
 * having to jump through FFI hoops with Rust.
 */

#include <string.h>
#include <stdio.h>
#include <sys/resource.h>
#include <sys/time.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

double rtime(void) {
  struct timespec tp;
  clock_gettime(CLOCK_MONOTONIC, &tp);
  return tp.tv_sec + tp.tv_nsec * 1e-9;
}

int main(int argc, char *argv[]) {
  pid_t pid;
  double start = rtime();
  pid = fork();
  if (pid < 0) { // the fork failed
    perror(NULL);
    return -1;
  }
  if (pid != 0) { // the parent process receives the child's pid
    int status;
    struct rusage r;
    pid_t ret;

    // wait for the child to exit, and ask the kernel for usage stats
    while ((ret = wait3(&status, 0, &r)) != pid) {
      if (ret == -1) {
        perror(NULL);
        return -1;
      }
    }

    double real_time = rtime() - start;

    printf("=== stats ===\n");
    if (WIFEXITED(status)) {
      printf("exit code: %d\n", WEXITSTATUS(status));
    }
    if (WIFSIGNALED(status)) {
      int sig = WTERMSIG(status);
      printf("signal:    (%d) %s\n", sig, strsignal(sig));
    }

    printf("real:      %.9f\n", real_time);
    printf("user:      %.9f\n", r.ru_utime.tv_sec + 1e-6 * r.ru_utime.tv_usec);
    printf("sys:       %.9f\n", r.ru_stime.tv_sec + 1e-6 * r.ru_stime.tv_usec);
  } else { // the child process has pid == 0
    execvp(argv[1], &argv[1]);
    // execvp shouldn't ever return, so if we reached here something went wrong
    perror(NULL);
  }
  return 0;
}