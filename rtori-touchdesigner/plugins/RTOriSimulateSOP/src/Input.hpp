#pragma once

#include <string_view>
namespace rtori_td {

struct Input {
  public:
	int64_t inputNumber;

	std::string_view fold;
	bool extractPosition;
	bool extractError;
	bool extractNormals;
	bool extractVelocity;

	static constexpr bool DEFAULT_EXTRACT_POSITIONS = true;
	static constexpr bool DEFAULT_EXTRACT_ERROR = false;
	static constexpr bool DEFAULT_EXTRACT_NORMALS = false;
	static constexpr bool DEFAULT_EXTRACT_VELOCITY = false;
};

} // namespace rtori_td