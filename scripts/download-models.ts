#!/usr/bin/env tsx
/**
 * Script to download models for AiMitra
 * Usage: npm run download:models
 */

import { MODEL_REGISTRY } from '../src/lib/models'
import { createWriteStream } from 'fs'
import { pipeline } from 'stream/promises'
import { createHash } from 'crypto'

const MODELS_DIR = './models'

async function downloadModel(model: typeof MODEL_REGISTRY[0]) {
  console.log(`\n📥 Downloading ${model.name} (${model.sizeGB} GB)...`)

  const response = await fetch(model.url)
  if (!response.ok) {
    throw new Error(`Failed to download: ${response.statusText}`)
  }

  const totalSize = Number(response.headers.get('content-length') || 0)
  let downloaded = 0

  const filePath = `${MODELS_DIR}/${model.filename}`
  const fileStream = createWriteStream(filePath)
  const hash = createHash('sha256')

  const reader = response.body!.getReader()

  while (true) {
    const { done, value } = await reader.read()
    if (done) break

    downloaded += value.length
    hash.update(value)
    fileStream.write(value)

    const progress = totalSize > 0 ? ((downloaded / totalSize) * 100).toFixed(1) : '?'
    process.stdout.write(`\r  Progress: ${progress}% (${(downloaded / 1024 / 1024).toFixed(1)} MB / ${(totalSize / 1024 / 1024).toFixed(1)} MB)`)
  }

  fileStream.end()
  await new Promise(resolve => fileStream.on('finish', resolve))

  const actualHash = hash.digest('hex')
  console.log(`\n  SHA256: ${actualHash}`)

  if (model.sha256 !== '...' && model.sha256 !== 'a1b2c3d4e5f6...') {
    if (actualHash !== model.sha256) {
      throw new Error(`Checksum mismatch! Expected ${model.sha256}, got ${actualHash}`)
    }
    console.log('  ✅ Checksum verified')
  }

  console.log(`  ✅ Saved to ${filePath}`)
}

async function main() {
  console.log('🔧 AiMitra Model Downloader')
  console.log('================================')

  // Create models directory
  await Bun.write(`${MODELS_DIR}/.gitkeep`, '')
  console.log(`\n📁 Models directory: ${MODELS_DIR}`)

  // Download recommended models
  const recommended = MODEL_REGISTRY.filter(m => m.recommended)

  for (const model of recommended) {
    try {
      await downloadModel(model)
    } catch (error) {
      console.error(`\n❌ Failed to download ${model.name}:`, error)
    }
  }

  console.log('\n✨ Done!')
}

main().catch(console.error)