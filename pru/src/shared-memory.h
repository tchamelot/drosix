#pragma once

enum debug_config {
  DEBUG_CONFIG_NONE,
  DEBUG_CONFIG_PID_LOOP,
  DEBUG_CONFIG_PID_NEW_DATA,
  DEBUG_CONFIG_PWM_STEP,
  DEBUG_CONFIG_PWM_CHANGE,
};

struct pid_config {
  float kpa;
  float kpr;
  float ti;
  float td;
  float filter;
  float kaw;
  float max;
  float min;
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

typedef uint32_t u32;

typedef struct pid_config pid_config_t;

typedef struct odometry odometry_t;

typedef struct angles angles_t;

typedef enum debug_config debug_config_t;

struct shared_mem {
  u32 period;
  pid_config_t pid_roll;
  pid_config_t pid_pitch;
  pid_config_t pid_yaw;
  pid_config_t pid_thrust;
  odometry_t pid_input;
  u32 pid_output[4];
  angles_t p_pid;
  angles_t v_pid;
  u32 cycle;
  u32 stall;
  debug_config_t debug_config;
};
