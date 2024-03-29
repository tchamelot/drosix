#pragma once

struct pid_controller_t {
    float a[3];
    float b[2];
    float inputs[2];
    float outputs[2];
};

void pid_init(struct pid_controller_t* pid, const volatile float a[3], const volatile float b[2]);

void pid_reset(struct pid_controller_t* pid);

float pid_run(struct pid_controller_t* pid, float input);
