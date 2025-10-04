#pragma once

#include "rtori/Context.hpp"
#include "rtori/Solver.hpp"

#include <iterator>
#include <optional>
#include <format>
#include <rtori/FoldFileParseError.d.hpp>
#include <rtori/FoldFileParseErrorKind.d.hpp>
#include <rtori/JSONParseError.d.hpp>

namespace rtori::rtori_td {

enum class SolverImportResultKind {
	Success,
	SolverCreationError,
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

	std::string format() const;
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

	bool isLoaded() const;
};

} // namespace rtori::rtori_td