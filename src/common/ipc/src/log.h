#ifndef LOG_H
#define LOG_H

#include <stdio.h>

#define RED "\x1B[31m"
#define GRN "\x1B[32m"
#define YEL "\x1B[33m"
#define BLU "\x1B[34m"
#define MAG "\x1B[35m"
#define CYN "\x1B[36m"
#define WHT "\x1B[37m"
#define RST "\x1B[0m"

#define LOG_DEBUG(fmt, ...)                                                \
  fprintf(stderr, GRN "[DEBUG] " RST __FILE__ ": " fmt "\n" __VA_OPT__(, ) \
                      __VA_ARGS__)

#define LOG_INFO(fmt, ...) \
  fprintf(stderr,          \
          BLU "[INFO] " RST __FILE__ ": " fmt "\n" __VA_OPT__(, ) __VA_ARGS__)

#define LOG_WARN(fmt, ...) \
  fprintf(stderr,          \
          YEL "[WARN] " RST __FILE__ ": " fmt "\n" __VA_OPT__(, ) __VA_ARGS__)

#define LOG_ERR(fmt, ...) \
  fprintf(stderr,         \
          RED "[ERR] " RST __FILE__ ": " fmt "\n" __VA_OPT__(, ) __VA_ARGS__)

#endif
