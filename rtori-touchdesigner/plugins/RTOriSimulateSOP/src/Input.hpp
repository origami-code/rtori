#pragma once

#include <string>

namespace rtori_td {

template<typename T> struct InputChangeWrapper {
  public:
	T value;
	bool changed;

	template<typename U> InputChangeWrapper<T> update(U newValue) const {
		if (this->value != newValue) {
			return InputChangeWrapper<T>{.value = T(newValue), .changed = true};
		} else {
			return InputChangeWrapper<T>{.value = this->value, .changed = false};
		}
	}

	static InputChangeWrapper<T> create(T newValue) {
		return InputChangeWrapper<T>{.value = newValue, .changed = true};
	}
};

struct Input {
  public:
	int64_t inputNumber;

	/// A copy of the input string
	InputChangeWrapper<std::string> foldFileSource;
	InputChangeWrapper<int16_t> frameIndex;

	InputChangeWrapper<bool> extractPosition;
	InputChangeWrapper<bool> extractError;
	InputChangeWrapper<bool> extractVelocity;

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

} // namespace rtori_td