#pragma once

#include <optional>
#include <vector>

namespace rtori_td {

struct Output {
  public:
	int64_t outputNumber;

	std::vector<int32_t> indices;

	std::vector<float> backingBuffer;
	std::optional<std::tuple<int32_t, int32_t>> positions;
	std::optional<std::tuple<int32_t, int32_t>> normals;
	std::optional<std::tuple<int32_t, int32_t>> error;
	std::optional<std::tuple<int32_t, int32_t>> velocity;

	std::optional<float> maxError;
	std::optional<float> maxVelocity;

	float dt;
	float simulationTime;
	int32_t stepsPerCook;

	bool validInput;
};

} // namespace rtori_td