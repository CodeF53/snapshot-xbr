interface VersionInfo {
  id: string
  type: 'snapshot' | 'release'
  url: string
  time: string
  releaseTime: string
}
interface VersionManifest {
  latest: { release: string, snapshot: string }
  versions: VersionInfo[]
}
export async function getVersionManifest() {
  return await fetch('https://launchermeta.mojang.com/mc/game/version_manifest.json').then(r => r.json()) as VersionManifest
}

interface DownloadInfo { sha1: string, size: number, url: string }
export interface VersionMeta {
  arguments: unknown
  assetIndex: unknown
  assets: string
  complianceLevel: number
  downloads: {
    client: DownloadInfo
    client_mappings: DownloadInfo
    server: DownloadInfo
    server_mappings: DownloadInfo
  }
  id: string
  javaVersion: unknown
  libraries: unknown
  logging: unknown
  mainClass: string
  minimumLauncherVersion: number
  releaseTime: string
  time: string
  type: string
}
