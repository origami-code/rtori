#include "Solver.hpp"
#include <rtori_core.hpp>
#include <cassert>
#include <iostream>

using namespace rtori::rtori_td;

Solver::Solver(rtori::Context const* context)
	: solver(nullptr), foldFile(nullptr), frameIndex(0), transformedData(nullptr) {
	constexpr rtori::Parameters solverCreationParams = {.solver =
														  rtori::SolverKind::OrigamiSimulator,
														.backend = rtori::BackendFlags_ANY};

	this->solver = rtori::rtori_ctx_create_solver(context, &solverCreationParams);
}

Solver::~Solver() {
	if (this->transformedData != nullptr) {
		rtori::rtori_fold_transformed_drop(this->transformedData);
		this->transformedData = nullptr;
	}

	if (this->foldFile != nullptr) {
		rtori::rtori_fold_deinit(this->foldFile);
		this->foldFile = nullptr;
	}

	rtori::rtori_solver_deinit(this->solver);
	this->solver = nullptr;
}

SolverImportResult Solver::update(std::optional<std::string_view> fold,
								  std::optional<uint16_t> frameIndex,
								  std::optional<float> foldPercentage) {
	assert(this->solver != nullptr);
	rtori::Context const* context = rtori_solver_get_context(this->solver);

	if (fold.has_value()) {
		const std::string_view foldInner = fold.value();
		const rtori::FoldFile* candidateFoldFile;

		if (foldInner.length() == 0) {
			candidateFoldFile = nullptr;
		} else {
			// First, parse
			const rtori::FoldParseResult foldParseResult =
			  rtori::rtori_fold_parse(context,
									  reinterpret_cast<const uint8_t*>(foldInner.data()),
									  foldInner.length());

			if (foldParseResult.status != rtori::FoldParseStatus::Success) {
				std::cout << "Error while parsing fold file" << std::endl;

				if (foldParseResult.status == rtori::FoldParseStatus::Error) {
					return SolverImportResult{
					  .kind = SolverImportResultKind::FoldParseError,
					  .payload{.parseError = foldParseResult.payload.error}};
				} else {
					assert(false);
				}
			}
			std::cout << "Parsed fold file" << std::endl;

			candidateFoldFile = foldParseResult.payload.file;
		}

		if (this->foldFile != nullptr) {
			if (this->transformedData != nullptr) {
				rtori::rtori_fold_transformed_drop(this->transformedData);
				this->transformedData = nullptr;
			}
			rtori::rtori_fold_deinit(this->foldFile);
		}
		this->foldFile = candidateFoldFile;
	}

	if (frameIndex.has_value()) {
		this->frameIndex = frameIndex.value();
	}

	if (this->foldFile != nullptr && (fold.has_value() || frameIndex.has_value())) {
		// Transform
		rtori::TransformedData* transformed =
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
	}

	if (this->foldFile == nullptr) {
		return SolverImportResult{
		  .kind = SolverImportResultKind::FoldEmpty,
		};
	} else {
		if (foldPercentage.has_value()) {
			SolverOperationResult result =
			  rtori::rtori_solver_set_fold_percentage(solver, foldPercentage.value());
			assert(result == SolverOperationResult::Success);
		}

		// Done
		return SolverImportResult{.kind = SolverImportResultKind::Success};
	}
}