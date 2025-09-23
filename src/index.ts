import type { VersionMeta } from './mojangApi'
import { copyFile, mkdir } from 'node:fs/promises'
import { dirname } from 'node:path'
import { $, Glob } from 'bun'
import { getVersionManifest } from './mojangApi'

const versionManifest = await getVersionManifest()
const versionInfo = versionManifest.versions[0]!
const versionMeta = await fetch(versionInfo.url).then(r => r.json()) as VersionMeta
const versionBlob = await fetch(versionMeta.downloads.client.url)

const versionAssetsDir = `./raw-assets/${versionInfo.id}`
unpackVersionAssets: {
  const bsdtarArgs = ['-xf', '-', '-C', versionAssetsDir]
  const nonTexturePatterns = [
    '*.class',
    '*.json',
    '.mcassetsroot',
    'pack.png',
    'shaders',
    'texts',
    'data',
    'META-INF',
    'flightrecorder-config.jfc',
  ]
  // exclude fonts/colormaps/title art/various textures that arent pixel art
  const nonPixelArtDirectoryPatterns = ['font', 'colormap', 'gui/title', 'gui/realms', 'misc']
  const nonPixelArtTextures = ['clouds', 'end_flash', 'end_sky', 'dither', 'isles'].map(name => `${name}.png`)
  const excludePatterns = [...nonTexturePatterns, ...nonPixelArtDirectoryPatterns, ...nonPixelArtTextures]

  for (const pattern of excludePatterns)
    bsdtarArgs.push('--exclude', pattern)
  await mkdir(versionAssetsDir, { recursive: true })

  const unzipProc = Bun.spawn(
    ['bsdtar', ...bsdtarArgs], // nish libarchive
    { stdin: versionBlob },
  )
  await unzipProc.exited
}

const outputDir = `./output/${versionInfo.id}`
await mkdir(versionAssetsDir, { recursive: true })

const assetGlob = new Glob('**/*')
for await (const path of assetGlob.scan(versionAssetsDir)) {
  const srcPath = `${versionAssetsDir}/${path}`
  const dstPath = `${outputDir}/${path}`
  await mkdir(dirname(dstPath), { recursive: true })

  // copy over mcmetas used for animations/nineslice
  if (path.endsWith('.mcmeta')) {
    await copyFile(srcPath, dstPath)
    continue
  }
  if (!path.endsWith('.png')) {
    console.error(`Unexpected non-image file ${path}`)
    continue
  }

  const settings = {
    wrap: false,
    relayer: false,
  }
  if (/\/(?:block|optifine|painting)\//.exec(path))
    settings.wrap = true
  else if (/\/(?:model|entity)\//.exec(path))
    settings.relayer = true

  await copyFile(srcPath, dstPath)

  // we cull all translucent pixels from the texture unless the src texture had translucent pixels by default
  const shouldCull = true // TODO: check if src texture has translucent pixels

  if (settings.wrap) {
    // TODO: add wrap to texture edges
  }

  await $`xbrzscale 4 ${dstPath} ${dstPath}`.quiet()

  if (settings.wrap) {
    // TODO: remove wrap from scaled texture edges
  }

  if (shouldCull) {
    // TODO: if (pixel.opacity < 191) pixel.opacity = 0; else pixel.opacity = 255
  }

  if (settings.relayer) {
    // TODO: relayer
  }
}
