#pragma once
#include "shared-memory.h"

struct pid_controller_t {
    float kd[3];
    float ki;
    float kaw;
    float max;
    float min;
    float d_out_prev;
    float i_out_prev;
    float input_prev;
    float sat_err_prev[2];
};

void pid_init(struct pid_controller_t* pid, const volatile pid_config_t* config, const float sampling);

void pid_reset(struct pid_controller_t* pid);

float pid_run(struct pid_controller_t* pid, float input);
