#include "Solver.hpp"
#include <cstdio>
#include <exception>
#include <rtori/BackendFlags.d.hpp>
#include <rtori/Solver.hpp>
#include <cassert>
#include <iostream>
#include <stdexcept>

#if __has_include(<format>)
#include <format>
namespace fmt = std;
#else
#include <fmt/format.h>
#endif

using namespace rtori::rtori_td;

std::string SolverImportResult::format(void) const {
	switch (kind) {
	case SolverImportResultKind::FoldParseError: {
		rtori::FoldFileParseError const& details = this->payload.parseError;
		if (details.status == rtori::FoldFileParseErrorKind::Empty) {
			return std::string("[ERROR] Couldn't parse fold file as it was empty");
		} else if (details.status == rtori::FoldFileParseErrorKind::Error) {
			rtori::JSONParseError const& jsonParseError = details.error.value();

			return fmt::format("[ERROR] Fold parse error \"{}\" on line {}, column {}",
							   (int32_t)jsonParseError.category,
							   jsonParseError.line,
							   jsonParseError.column);
		} else {
			return std::string("[ERROR] Unknown fold parse error");
		}
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

Solver::Solver(std::shared_ptr<rtori::Context> context)
	: context(context), solver(nullptr), foldFile(nullptr), frameIndex(0) {
	rtori::Parameters const solverCreationParams = {.family =
													  rtori::SolverFamily::OrigamiSimulator,
													.backend = rtori::BackendFlags::any()};

	auto&& result = context->create_solver_sync(solverCreationParams);
	if (result.is_err()) {
		auto err = std::move(result).err().value().format();
		throw std::runtime_error(err);
	}

	solver = std::move(result).ok().value();
}

Solver::~Solver() {}

SolverImportResult Solver::update(std::optional<std::string_view> fold,
								  std::optional<uint16_t> frameIndex,
								  std::optional<float> foldPercentage) {
	assert(this->solver != nullptr);

	if (fold.has_value()) {
		const std::string_view foldInner = fold.value();
		std::unique_ptr<rtori::FoldFile> candidateFoldFile;

		if (foldInner.length() == 0) {
			candidateFoldFile = nullptr;
		} else {
			// First, parse
			auto& context = *this->context.get();
			auto foldParseResult = rtori::FoldFile::parse_bytes(
			  context,
			  diplomat::span<const uint8_t>(reinterpret_cast<const uint8_t*>(foldInner.data()),
											foldInner.length()));

			if (foldParseResult.is_err()) {
				std::cout << "Error while parsing fold file" << std::endl;

				auto err = std::move(foldParseResult).err().value();

				return SolverImportResult{.kind = SolverImportResultKind::FoldParseError,
										  .payload{.parseError = err}};
			}

			std::cout << "Parsed fold file" << std::endl;

			candidateFoldFile = std::move(foldParseResult).ok().value();
		}

		if (this->foldFile != nullptr) {
			// if (this->transformedData != nullptr) {
			//	rtori::rtori_fold_transformed_drop(this->transformedData);
			//	this->transformedData = nullptr;
			// }
			// rtori::rtori_fold_deinit(this->foldFile);
		}
		this->foldFile = std::move(candidateFoldFile);
	}

	if (frameIndex.has_value()) {
		this->frameIndex = frameIndex.value();
	}

	if (this->foldFile != nullptr && (fold.has_value() || frameIndex.has_value())) {
		#if 0
		// TODO: Cache by solver family
		// So, TransformationCache which contains, for example, OSCache, which contains the
		// supplemented data
		// Transform
		rtori::SupplementedInput* transformed =
		  rtori::rtori_fold_transform(this->foldFile, this->frameIndex);
		if (this->transformedData != nullptr) {
			rtori::rtori_fold_transformed_drop(this->transformedData);
		}
		this->transformedData = transformed;

		// Then load
		const rtori::SolverOperationResult solverLoadResult =
		  rtori::rtori_solver_load_from_transformed(solver, transformed);
		if (solverLoadResult != rtori::SolverOperationResult::Success) {
			std::cout << "Error while loading fold file" << std::endl;

			return SolverImportResult{.kind = SolverImportResultKind::FoldLoadError};
		}
		std::cout << "Loaded fold file" << std::endl;
		#else
		
		// The solver is created from the context
		this->solver->load_from_fold(*this->foldFile, this->frameIndex);
		std::cout << "Loaded fold file" << std::endl;

		#endif
	}

	if (this->foldFile == nullptr) {
		return SolverImportResult{
		  .kind = SolverImportResultKind::FoldEmpty,
		};
	} else {
		if (foldPercentage.has_value()) {
			const auto result = solver->set_fold_percentage(foldPercentage.value());
			assert(result.is_ok());
		}

		// Done
		return SolverImportResult{.kind = SolverImportResultKind::Success};
	}
}

bool Solver::isLoaded() const {
	return this->solver->loaded();
}