#pragma once

#include <stdint.h>

enum debug_config {
  NoDebug = 0,
  PidLoop = 1,
  PidNewData = 2,
  PwmStep = 4
};

struct pid {
  float numerator[3];
  float denominator[2];
};

struct angle_pid {
  struct pid roll;
  struct pid pitch;
  struct pid yaw;
};

struct angles {
  float roll;
  float pitch;
  float yaw;
};

struct odometry {
  struct angles attitude;
  struct angles rate;
  float thrust;
};

typedef struct angle_pid angle_pid_t;

typedef struct pid pid_parameter_t;

typedef enum debug_config debug_config_t;

typedef struct odometry odometry_t;

typedef uint32_t u32;

typedef struct angles angles_t;

struct pru_shared_mem {
  angle_pid_t attitude_pid;
  pid_parameter_t thrust_pid;
  angle_pid_t rate_pid;
  debug_config_t debug_config;
  odometry_t pid_input;
  u32 pid_output[4];
  angles_t p_pid;
  angles_t v_pid;
  u32 cycle;
  u32 stall;
};
