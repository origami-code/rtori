#ifndef RTORI_TD_SIMULATE_OP_HPP_
#define RTORI_TD_SIMULATE_OP_HPP_

#include "Simulator.hpp"

#include <memory>

namespace rtori::rtori_td {

/// Abstract class that should be implemented by every OP (SOP, TOP, ...) implementing a Simulate.. class OP
class SimulateOP {
  public:
	/// The returned `shared_ptr` might be empty at initialization time
	virtual std::shared_ptr<Simulator> simulator();
};

} // namespace rtori::rtori_td

#endif // !RTORI_TD_SIMULATE_OP_HPP_
