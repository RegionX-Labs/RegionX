# RegionX

## 1. Introduction

[RegionX](https://regionx.tech/) is a project dedicated to developing components for the new Agile Coretime model. The goal of the project is to enable developer teams, researchers, and speculators to start trading, tracking, and analyzing the product Polkadot offers - Coretime. 

This repository is establishing the smart contract components for creating a secondary Coretime market. This infrastructure is meant to be leveraged by any end-user product built around Coretime.

The repository currently contains two crucial components in the form of ink! contracts that are necessary for the creation of a flexible and decentralized Coretime market.

## 2. Core components

### 2.1 Cross-Chain Regions

From a procurement perspective, regions can be seen as NFT tokens representing ownership of Coretime. Each region is characterized by a defined set of attributes that encompass all its properties. The following is the list of these attributes:

- `begin`: Specifies the starting point of time from which a task assigned to the region can be scheduled on a core.
- `end`: Specifies the deadline until when a task assigned to the region can be scheduled on a core.
- `length`: The duration of a region. Always equals to `end - begin`.
- `core`: The core index to which the region belongs.
- `part`: The maximum core resources the region can utilize within a relay chain block.
- `owner`: The owner of the region.
- `paid`: The payment for the region on the bulk market. Defined only for renewable regions. This is used to calculate the renewal price in the next bulk sale.

### 2.2 Coretime Marketplace

## 3. Develop
