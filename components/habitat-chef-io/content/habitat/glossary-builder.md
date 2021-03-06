+++
title = "Builder"
description = "Builder"

[menu]
  [menu.habitat]
    title = "Builder"
    identifier = "habitat/overview/concepts/builder"
    parent = "habitat/overview/concepts"
+++

Chef Habitat Builder consists of the Builder service and the Depot.

## Chef Habitat Builder

Users have the option to connect their GitHub repositories to Builder to enable continuous builds of their plans. Checking in new code to GitHub initiates a new build through a GitHub hook. If you've added your Chef Habitat plan to the root of your source code repository and your project depends on any of the Chef Habitat Core Packages (for example, openssl, ruby, or node), when these packages are updated, Builder automatically rebuilds your software and posts the updated package to your project's `unstable` channel, where it will wait until you review and promote it according to your regular release procedure.

## Builder on-prem

The Depot is a searchable repository that stores artifacts for use in Chef Habitat.


