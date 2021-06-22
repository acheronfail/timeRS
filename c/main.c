/**
 * This is just a proof of concept/scratch file used to prototype C APIs without
 * having to jump through FFI hoops with Rust.
 */

#include <stdio.h>
#include <string.h>
#include <sys/resource.h>
#include <sys/time.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

const char *humanSize(uint64_t bytes) {
  char *suffix[] = {"B", "KB", "MB", "GB", "TB"};
  char length = sizeof(suffix) / sizeof(suffix[0]);

  int i = 0;
  double dblBytes = bytes;

  if (bytes > 1024) {
    for (i = 0; (bytes / 1024) > 0 && i < length - 1; i++, bytes /= 1024) {
      dblBytes = bytes / 1024.0;
    }
  }

  static char output[200];
  sprintf(output, "%.04lf %s", dblBytes, suffix[i]);
  return output;
}

int32_t n_cpu(void) { return (int32_t)sysconf(_SC_NPROCESSORS_ONLN); }
int32_t page_size(void) { return (int32_t)sysconf(_SC_PAGESIZE); }
int64_t mem_total(void) {
  return (int64_t)sysconf(_SC_PHYS_PAGES) * page_size();
}

#if defined(__APPLE__)
// TODO: review -> it's not what other applications (top, istatmenus, etc) report
// may need to take into account stolen pages
#include <mach/mach.h>
int64_t mem_avail(void) {
  natural_t c = HOST_VM_INFO64_COUNT;
  struct vm_statistics64 v;
  struct vm_statistics w;
  printf("SIZE: %d\n", HOST_VM_INFO64_COUNT);
  if (host_statistics64(mach_host_self(), HOST_VM_INFO64, (host_info64_t)&v,
                        &c) != KERN_SUCCESS) {
    return -1;
  }

  return (int64_t)(v.external_page_count + v.purgeable_count + v.free_count -
                   v.speculative_count) *
         page_size();
}
#elif defined(__linux__)
// TODO
int64_t mem_avail(void) { return -1; }
#else
int64_t mem_avail(void) { return -1; }
#endif

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
    printf("n_cpu:     %d\n", n_cpu());
    printf("mem_avail: %lld (%s)\n", mem_avail(), humanSize(mem_avail()));
    printf("mem_total: %lld (%s)\n", mem_total(), humanSize(mem_total()));
  } else { // the child process has pid == 0
    execvp(argv[1], &argv[1]);
    // execvp shouldn't ever return, so if we reached here something went wrong
    perror(NULL);
  }
  return 0;
}