#!/usr/bin/env bash

echo "Updating project..."
git pull
npm ci

echo "Starting build"
npm run build
