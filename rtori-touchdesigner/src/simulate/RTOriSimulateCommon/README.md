# RTOriSimulateCommon: common code & interfaces to be shared between all Simulate OPs

Two main classes:
- `rtori::rtori_td::Simulator`: class implementing most of the OPerator-Type-Agnostic functions for consuming OPs
    - e.g definining common simulation parameters, info CHOPs, etc.
- `rtori::rtori_td::SimulateOP`: abstract class that should be implemented by every Simulate OPerator
    - allows sharing the same simulation code even when one wants an SOP + a TOP output based on the same input
    - input needs to be stricly the same