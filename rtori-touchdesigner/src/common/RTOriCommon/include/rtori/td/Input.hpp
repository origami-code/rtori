#pragma once

#include <chrono>
#include <string>

namespace rtori::rtori_td {

template<typename T> struct InputChangeWrapper {
  public:
	T value;
	bool changed;

	InputChangeWrapper(T value = T(), bool changed = true) : value(value), changed(changed) {}

	template<typename U> inline InputChangeWrapper<T> update(U newValue) const {
		std::equal_to<> equal_to{};
		bool changed = !equal_to(this->value, newValue);

		if (changed) {
			return InputChangeWrapper<T>(T(newValue), true);
		} else {
			return InputChangeWrapper<T>(this->value, false);
		}
	}
};

struct Input {
  public:
	int64_t inputNumber = 0;

	/// A copy of the input string
	InputChangeWrapper<std::string> foldFileSource;
	InputChangeWrapper<uint16_t> frameIndex;

	InputChangeWrapper<float> foldPercentage;

	InputChangeWrapper<bool> extractPosition;
	InputChangeWrapper<bool> extractError;
	InputChangeWrapper<bool> extractVelocity;

	InputChangeWrapper<float> timeScale = 1.0;
	InputChangeWrapper<bool> adaptive = false;
	InputChangeWrapper<float> frameBudget = 1.0;

	InputChangeWrapper<std::chrono::microseconds> targetPeriod;

	static constexpr bool DEFAULT_EXTRACT_POSITIONS = true;
	static constexpr bool DEFAULT_EXTRACT_ERROR = false;
	static constexpr bool DEFAULT_EXTRACT_NORMALS = false;
	static constexpr bool DEFAULT_EXTRACT_VELOCITY = false;

	/// An input has changed if any of the data members is marked as such
	inline bool changed() const {
		return foldFileSource.changed || frameIndex.changed || foldPercentage.changed ||
			   extractPosition.changed || extractError.changed || extractVelocity.changed ||
			   timeScale.changed || adaptive.changed || frameBudget.changed ||
			   targetPeriod.changed;
	}
};

} // namespace rtori::rtori_td