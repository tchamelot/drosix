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

void pid_init(struct pid_t* pid, float kp, float ki, float kd1, float kd2);

float pid_run(struct pid_t* pid, float error);
