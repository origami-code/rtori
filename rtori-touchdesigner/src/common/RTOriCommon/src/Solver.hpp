#pragma once

#include "rtori/Context.hpp"
#include "rtori/Solver.hpp"

#include <optional>
#include <format>
#include <rtori/FoldFileParseError.d.hpp>
#include <rtori/JSONParseError.d.hpp>

namespace rtori::rtori_td {

enum class SolverImportResultKind {
	Success,
	FoldEmpty,
	FoldParseError,
	FoldLoadError,
};

struct SolverImportResult final {
  public:
	SolverImportResultKind kind;
	union {
		rtori::FoldFileParseError parseError;
	} payload;

	std::string format() const {
		switch (kind) {
		case SolverImportResultKind::FoldParseError: {
			rtori::FoldFileParseError const& details = this->payload.parseError;
			// std::format requires Xcode 15.3 or later
			return /*std::format("[ERROR] Fold parse error \"{}\" on line {}, column {}",
							   (int32_t)details.category,
							   details.line,
							   details.column);*/ std::string("[ERROR] Fold parse error");
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

class Solver final {
  public:
	std::shared_ptr<rtori::Context> context;

	std::unique_ptr<rtori::Solver> solver;

	std::unique_ptr<rtori::FoldFile> foldFile;
	uint16_t frameIndex;

	// rtori::SupplementedInput* transformedData;

	Solver(std::shared_ptr<rtori::Context> ctx);
	~Solver();

	SolverImportResult update(std::optional<std::string_view> fold,
							  std::optional<uint16_t> frameIndex,
							  std::optional<float> foldPercentage);
};

} // namespace rtori::rtori_td