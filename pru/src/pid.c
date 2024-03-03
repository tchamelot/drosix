#include <string.h>
#include <stdint.h>
#include "pid.h"

void pid_init(struct pid_controller_t* pid, const volatile float a[3], const volatile float b[2]) {
    uint32_t i;
    for(i = 0u; i < 3; i++) {
      pid->a[i] = a[i];
    }
    for(i = 0u; i < 2; i++) {
      pid->b[i] = b[i];
    }
    memset(pid->inputs, 0, sizeof(float)*2);
    memset(pid->outputs, 0, sizeof(float)*2);
}

void pid_reset(struct pid_controller_t* pid) {
    memset(pid->inputs, 0, sizeof(float)*2);
    memset(pid->outputs, 0, sizeof(float)*2);
}

float pid_run(struct pid_controller_t* pid, float input) {
    float output = input*pid->a[0] + pid->inputs[0]*pid->a[1] + pid->inputs[1]*pid->a[2] - pid->outputs[0]*pid->b[0] - pid->outputs[1]*pid->b[1];

    pid->outputs[1] = pid->outputs[0];
    pid->outputs[0] = output;

    pid->inputs[1] = pid->inputs[0];
    pid->inputs[0] = input;

    return output;
}
