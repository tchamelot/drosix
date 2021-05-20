#pragma once

struct pid_t {
    float kp;
    float ki;
    float kd1;
    float kd2;
    float i_prev;
    float d_prev;
    float error_prev;
};

float run_pid(struct pid_t* pid, float error);
