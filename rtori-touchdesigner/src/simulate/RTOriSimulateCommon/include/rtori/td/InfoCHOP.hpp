#ifndef RTORI_TD_INFOCHOP_HPP_
#define RTORI_TD_INFOCHOP_HPP_

#include <cstdint>

constexpr int32_t INFO_CHOP_CHANNEL_LOADED_INDEX = 0;
constexpr const char* INFO_CHOP_CHANNEL_LOADED_NAME = "rtori_loaded";

constexpr int32_t INFO_CHOP_CHANNEL_DT_INDEX = 1;
constexpr const char* INFO_CHOP_CHANNEL_DT_NAME = "rtori_dt";

constexpr int32_t INFO_CHOP_CHANNEL_STEPS_PER_COOK_INDEX = 2;
constexpr const char* INFO_CHOP_CHANNEL_STEPS_PER_COOK_NAME = "rtori_steps_per_cook";

constexpr int32_t INFO_CHOP_CHANNEL_SIMULATION_TIME_INDEX = 3;
constexpr const char* INFO_CHOP_CHANNEL_SIMULATION_TIME_NAME = "rtori_simulation_time";

constexpr int32_t INFO_CHOP_CHANNEL_TOTAL_STEP_TIME_INDEX = 4;
constexpr const char* INFO_CHOP_CHANNEL_TOTAL_STEP_TIME_NAME = "rtori_total_step_time";

constexpr int32_t INFO_CHOP_CHANNEL_EXTRACT_TIME_INDEX = 5;
constexpr const char* INFO_CHOP_CHANNEL_EXTRACT_TIME_NAME = "rtori_extract_time";

constexpr int32_t INFO_CHOP_CHANNEL_RUNNING_INDEX = 6;
constexpr const char* INFO_CHOP_CHANNEL_RUNNING_NAME = "rtori_running";

constexpr int32_t INFO_CHOP_CHANNEL_MAX_VELOCITY_INDEX = 7;
constexpr const char* INFO_CHOP_CHANNEL_MAX_VELOCITY_NAME = "rtori_max_velocity";

constexpr int32_t INFO_CHOP_CHANNEL_MAX_ERROR_INDEX = 8;
constexpr const char* INFO_CHOP_CHANNEL_MAX_ERROR_NAME = "rtori_max_error";

constexpr const char* INFO_CHOP_CHANNEL_NAMES[]{INFO_CHOP_CHANNEL_LOADED_NAME,
												INFO_CHOP_CHANNEL_DT_NAME,
												INFO_CHOP_CHANNEL_STEPS_PER_COOK_NAME,
												INFO_CHOP_CHANNEL_SIMULATION_TIME_NAME,
												INFO_CHOP_CHANNEL_TOTAL_STEP_TIME_NAME,
												INFO_CHOP_CHANNEL_EXTRACT_TIME_NAME,
												INFO_CHOP_CHANNEL_RUNNING_NAME,
												INFO_CHOP_CHANNEL_MAX_VELOCITY_NAME,
												INFO_CHOP_CHANNEL_MAX_ERROR_NAME};

#endif /* RTORI_TD_INFOCHOP_HPP_ */