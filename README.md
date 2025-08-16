# Exdsdevs framework

The **exdsdevs** is a multi-modeling and simulation library, written in rust. It is a modeler and simulator supporting the use of devs formalism for models specification and simulation.

Exdsdevs is based on the theory of modeling and simulation initially developed by B.P. Zeigler in the 70’s and continuously enriched until now by an active international community.

Exdsdevs is based on the DEVS formalism (Discrete Event systems Specification). Exdsdevs provides a set of rust libraries and a simulator. The Exdsdevs are designed to allow the development of new simulators, models or new programs for modeling and analysis.

Our goal with Exdsdevs is to provide powerful tools for modeling, simulating and analysing complex dynamics systems. We hope build an easy to use software. Our development are complying with the [DEVS] specification [DEVS] and works made by the simulation community.

Exdsdevs is a free environment of multi-modelling and simulation developed under the [licence MIT or Apache 2.0]. All source code are available on Github.

## What's DEVS

DEVS, Discrete Event System Specification is a modular and hierarchic formalism for modelling, simulation and study of complex systems. These system can be discrete event systems described by state transition functions or continuous systems described by differential equation for instance or hybrid systems.

## Exdsdevs

In Exdsdevs, we have implemented the abstract simulator based on the article by Hagendorf, Olaf & Pawletta, Thorsten & Deatcu, Christina. (2009). Extended dynamic structure DEVS. 21st European Modeling and Simulation Symposium, EMSS 2009. <https://www.researchgate.net/publication/288697606_Extended_dynamic_structure_DEVS>. The simulator has been improved in terms of unification of model types. In contrast to the classical DEVS, which has two types of models (Coupled and Atomic), the library implements one type of models. We also introduced an simple observation framework in the DEVS simulator of Exdsdevs.
