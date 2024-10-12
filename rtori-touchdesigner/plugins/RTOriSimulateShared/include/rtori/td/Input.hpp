#pragma once

#include <string>

namespace rtori::rtori_td {

template<typename T> struct InputChangeWrapper {
  public:
	T value;
	bool changed;

	InputChangeWrapper(T value = T(), bool changed = true) : value(value), changed(true) {}

	template<typename U> InputChangeWrapper<T> update(U newValue) const {
		if (this->value != newValue) {
			return InputChangeWrapper<T>(T(newValue), true);
		} else {
			return InputChangeWrapper<T>(this->value, false);
		}
	}
};

enum class PackingTiming {
	OnDemand,
	Prepack,
};

struct Input {
  public:
	int64_t inputNumber;

	/// A copy of the input string
	InputChangeWrapper<std::string> foldFileSource;
	InputChangeWrapper<uint16_t> frameIndex;

	InputChangeWrapper<float> foldPercentage;

	InputChangeWrapper<bool> extractPosition;
	InputChangeWrapper<bool> extractError;
	InputChangeWrapper<bool> extractVelocity;

	/// If the speedTarget is 0, then it will run as fast as possible
	/// Otherwise, this is the speed of the simulation
	InputChangeWrapper<float> speedTarget;

	InputChangeWrapper<PackingTiming> packingTiming;

	static constexpr bool DEFAULT_EXTRACT_POSITIONS = true;
	static constexpr bool DEFAULT_EXTRACT_ERROR = false;
	static constexpr bool DEFAULT_EXTRACT_NORMALS = false;
	static constexpr bool DEFAULT_EXTRACT_VELOCITY = false;

	/// An input has changed if any of the data members is marked as such
	bool changed() const {
		return foldFileSource.changed || frameIndex.changed || extractPosition.changed ||
			   extractError.changed || extractVelocity.changed;
	}
};

} // namespace rtori::rtori_td