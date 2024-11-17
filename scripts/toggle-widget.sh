#!/bin/bash

busctl --user call io.syph.rrwidget1 /io/syph/rrwidget1 io.syph.rrwidget1 ToggleVisibility 2> /dev/null || exit 0
