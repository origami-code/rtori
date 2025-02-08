#pragma once

#include "rtori_core.hpp"

#include <optional>
#include <format>

namespace rtori::rtori_td {

enum class SolverImportResultKind {
	Success,
	FoldEmpty,
	FoldParseError,
	FoldLoadError,
};

struct SolverImportResult {
  public:
	SolverImportResultKind kind;
	union {
		rtori::JsonParseError parseError;
	} payload;

	std::string format() const {
		switch (kind) {
		case SolverImportResultKind::FoldParseError: {
			rtori::JsonParseError const& details = this->payload.parseError;
			return std::format("[ERROR] Fold parse error \"{}\" on line {}, column {}",
							   (int32_t)details.category,
							   details.line,
							   details.column);
		}
		case SolverImportResultKind::FoldLoadError:
			return std::string("[ERROR] Fold load error");
		case SolverImportResultKind::FoldEmpty:
			return std::string("[ERROR] Fold input is empty");
		case SolverImportResultKind::Success:
			return std::string("[SUCCESS] Fold loaded successfully");
		default:
			return std::string("[ERROR] Unknown error kind");
		}
	}
};

class Solver {
  public:
	rtori::Solver const* solver;

	rtori::FoldFile const* foldFile;
	uint16_t frameIndex;

	rtori::TransformedData* transformedData;

	Solver(rtori::Context const* ctx);
	~Solver();

	SolverImportResult update(std::optional<std::string_view> fold,
							  std::optional<uint16_t> frameIndex,
							  std::optional<float> foldPercentage);
};

} // namespace rtori::rtori_td