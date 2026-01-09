Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass -Force

param(
  [switch]$Continue
)

$ErrorActionPreference = "Stop"

function Assert-Admin {
  $id = [Security.Principal.WindowsIdentity]::GetCurrent()
  $p  = New-Object Security.Principal.WindowsPrincipal($id)
  if (-not $p.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw "Lance PowerShell en Administrateur."
  }
}

function Set-ResumeAfterReboot {
  $self = $MyInvocation.MyCommand.Path
  $cmd  = "powershell.exe -NoProfile -ExecutionPolicy Bypass -File `"$self`" -Continue"
  New-ItemProperty -Path "HKLM:\Software\Microsoft\Windows\CurrentVersion\RunOnce" `
    -Name "PrepClientWSL" -Value $cmd -PropertyType String -Force | Out-Null
}

function Enable-WSLFeature {
  # Etape 2 de ta liste
  Write-Host "[2/5] Activation de Microsoft-Windows-Subsystem-Linux (sans reboot immédiat)..."
  dism.exe /online /enable-feature /featurename:Microsoft-Windows-Subsystem-Linux /all /norestart | Out-Null

  # Check si reboot requis
  $feature = Get-WindowsOptionalFeature -Online -FeatureName Microsoft-Windows-Subsystem-Linux
  Write-Host ("    Feature state: {0}" -f $feature.State)

  # DISM ne dit pas toujours proprement "reboot requis" ici, donc on reboot quand même dans ton flow.
}

function Install-UbuntuWSL {
  # Etape 1 de ta liste
  Write-Host "[1/5] Installation de WSL + Ubuntu (si pas déjà présent)..."
  try {
    # Si Ubuntu déjà listée, pas besoin de réinstaller
    $list = wsl.exe -l -q 2>$null
    if ($list -and ($list -contains "Ubuntu")) {
      Write-Host "    Ubuntu déjà installée."
      return
    }
  } catch { }

  # Lance l'install. Ça peut installer aussi le moteur WSL si pas présent.
  # Si tu veux éviter le lancement auto, tu peux tester --no-launch, mais tu as demandé la commande simple.
  wsl.exe --install -d Ubuntu | Out-Null
}

function Reboot-Now {
  # Etape 3 de ta liste
  Write-Host "[3/5] Redémarrage..."
  Set-ResumeAfterReboot
  shutdown.exe /r /t 0
}

function Verify-Feature {
  # Etape 4 de ta liste
  Write-Host "[4/5] Vérification de la feature WSL..."
  $feature = Get-WindowsOptionalFeature -Online -FeatureName Microsoft-Windows-Subsystem-Linux
  $feature | Format-List | Out-String | Write-Host
}

function Set-WSL1Default {
  # Etape 5 de ta liste
  Write-Host "[5/5] Définir WSL 1 par défaut..."
  wsl.exe --set-default-version 1 | Out-Null
}

function Try-RootReady {
  Write-Host "[Post] Test: wsl -d Ubuntu -u root -- bash -c `"echo 'ready'`""
  try {
    $out = wsl.exe -d Ubuntu -u root -- bash -c "echo 'ready'"
    Write-Host "    Output: $out"
  } catch {
    Write-Host "    Échec du test root. Très probable que Ubuntu n'ait pas fini son init (création user) côté distro."
    Write-Host "    Dans ce cas, lance UNE fois: wsl -d Ubuntu (et crée un user), puis relance ce script."
  }
}

# ---------------- main ----------------
Assert-Admin

if (-not $Continue) {
  # 1) Install Ubuntu/WSL
  Install-UbuntuWSL

  # 2) Enable feature WSL (au cas où)
  Enable-WSLFeature

  # 3) Reboot (ton flow)
  Reboot-Now
  exit
}

# Après reboot
Verify-Feature
Set-WSL1Default

# Petit état pour debug
Write-Host "[Info] wsl --status"
try { wsl.exe --status } catch { Write-Host "    wsl --status non dispo/échoue, on continue." }

Write-Host "[Info] wsl -l -v"
try { wsl.exe -l -v } catch { Write-Host "    wsl -l -v échoue, on continue." }

Try-RootReady

Write-Host "Terminé."
