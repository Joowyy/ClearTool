# Paso 01 — Setup Azure Trusted Signing

**Área**: 12-distribution
**Tiempo estimado**: 2-3 horas activas + 1-3 días de espera (identity validation)
**Dependencias**: ninguna

## Qué hacemos

Crear cuenta de Azure Trusted Signing — el servicio de Microsoft que firma binarios con cert de organización válido por ~$10/mes. Es la opción más fácil para indie devs en 2026 (sustituye certificados físicos HSM/YubiKey).

## Por qué Azure Trusted Signing y no otras opciones

- **Self-signed (gratis)**: Windows SmartScreen bloquea. No vale.
- **OV cert clásico ($200/año)**: requiere "reputation building" (10-30 días de descargas para que SmartScreen confíe).
- **EV cert ($300+/año)**: requiere YubiKey física o cert en HSM. Caro y engorroso en CI.
- **Azure Trusted Signing ($10/mes)**: instant trust, integración GitHub Actions trivial, sin hardware.

## Pasos

### 1. Crear cuenta Azure

Si no tienes ya:
- Ve a https://azure.microsoft.com → "Sign up for free".
- Tarjeta de crédito requerida (no se cobra hasta usar paid services).

### 2. Crear "Trusted Signing Account"

En el portal Azure:
1. Buscar "Trusted Signing Accounts".
2. "+ Create".
3. Subscription: la tuya.
4. Resource group: crear nuevo `cleartool-rg`.
5. Account name: `cleartool-signing`.
6. Region: cualquiera cerca. East US es la más común.
7. SKU: **Basic** (suficiente para indie).
8. Review + Create. Tarda ~1 minuto.

### 3. Crear "Identity Validation"

Una vez creada la cuenta:
1. Abrir el account `cleartool-signing`.
2. Sidebar → "Identity Validations" → "+ New".
3. Tipo: **Public**.
4. Subject name: tu nombre legal o nombre de empresa que aparecerá en el cert ("Joel Sánchez Fernández" o "ClearTool Software").
5. Subject country: ES (o el tuyo).
6. Subject locality: tu ciudad.
7. Email: el de la cuenta Azure.
8. Submit.

**Microsoft valida la identidad en 1-3 días hábiles**. Pueden pedirte documentos (DNI/factura de empresa). Mientras esperas, sigue con los siguientes pasos del módulo (sin firma todavía).

### 4. Crear "Certificate Profile"

Tras aprobación de identity:
1. En el account, "Certificate Profiles" → "+ New".
2. Tipo: **Public**.
3. Identity Validation: la que acabas de aprobar.
4. Profile name: `cleartool-public`.
5. Submit. Tarda otros ~10 minutos en provisionarse.

### 5. Crear App Registration en Azure AD

Para que GitHub Actions pueda autenticarse:
1. Azure portal → "Microsoft Entra ID" (antes Azure AD) → "App registrations" → "+ New".
2. Name: `cleartool-ci`.
3. Supported account types: "Accounts in this organizational directory only".
4. Register.
5. Apunta los valores que verás: **Application (client) ID**, **Directory (tenant) ID**.

### 6. Crear Client Secret

En la App Registration recién creada:
1. Sidebar → "Certificates & secrets" → "Client secrets" → "+ New".
2. Description: "github-actions".
3. Expires: 12 meses (apunta en calendario para renovar).
4. Add.
5. **Copia el "Value" inmediatamente** — solo se muestra una vez.

### 7. Asignar permisos al App Registration

Volver a Trusted Signing Account → "Access control (IAM)" → "+ Add" → "Add role assignment":
- Role: **"Trusted Signing Certificate Profile Signer"**.
- Members: tu app `cleartool-ci`.
- Review + assign.

### 8. Verifica setup con AzureSignTool local

Para probar antes de meterlo en CI:

```powershell
# Instala AzureSignTool
dotnet tool install --global AzureSignTool

# Firma un binario de prueba
AzureSignTool sign `
  -tr http://timestamp.acs.microsoft.com `
  -kvu https://eus.codesigning.azure.net/ `
  -kvi "<tenant-id>" `
  -kvs "<client-secret>" `
  -kva "<client-id>" `
  -kvc cleartool-public `
  -tmd cleartool-signing `
  -fd sha256 `
  -v `
  "path\to\ClearTool.exe"
```

Si funciona, verifica:
```powershell
Get-AuthenticodeSignature "path\to\ClearTool.exe" | Format-List
# Status: Valid
# SignerCertificate: ... (subject = tu nombre)
```

### 9. Guardar secrets para GitHub Actions

En el repo GitHub → Settings → Secrets and variables → Actions → "New repository secret":

| Secret name | Value |
|-------------|-------|
| `AZURE_TENANT_ID` | el tenant ID de Azure AD |
| `AZURE_CLIENT_ID` | el application ID del app registration |
| `AZURE_CLIENT_SECRET` | el secret value que copiaste |
| `TRUSTED_SIGNING_ACCOUNT` | `cleartool-signing` |
| `TRUSTED_SIGNING_PROFILE` | `cleartool-public` |
| `TRUSTED_SIGNING_ENDPOINT` | `https://eus.codesigning.azure.net/` |

### 10. Configurar facturación

Azure Trusted Signing cuesta **$9.99/mes** (Basic SKU) + ~$0.005 por firma adicional. Activar billing alerts en Azure por encima de $20/mes.

## Criterio de done

- [ ] Trusted Signing Account creada.
- [ ] Identity Validation aprobada por Microsoft.
- [ ] Certificate Profile activo.
- [ ] App Registration con secret válido.
- [ ] Role assignment correcto.
- [ ] Firma local con AzureSignTool produce binario con `Status: Valid`.
- [ ] Secrets en GitHub Actions configurados.
- [ ] Billing alert configurado.

## Troubleshooting

- **"Forbidden: principal does not have permissions"** → no asignaste el role `Trusted Signing Certificate Profile Signer`.
- **"Identity validation pending"** → esperar a Microsoft (1-3 días).
- **"Timestamping failed"** → el `-tr` URL puede haber cambiado. Verifica en docs.

## Referencias

- [Azure Trusted Signing docs](https://learn.microsoft.com/en-us/azure/trusted-signing/)
- [Quickstart](https://learn.microsoft.com/en-us/azure/trusted-signing/quickstart)
- [AzureSignTool repo](https://github.com/vcsjones/AzureSignTool)
