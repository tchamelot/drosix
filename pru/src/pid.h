#pragma once

struct pid_t {
    float a[3];
    float b[2];
    float inputs[2];
    float outputs[2];
};

void pid_init(struct pid_t* pid, float a[3], float b[2]);

void pid_reset(struct pid_t* pid);

float pid_run(struct pid_t* pid, float input);
