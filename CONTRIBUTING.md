# Guía de Contribución (Spooler Agent SGI)

¡Bienvenido al desarrollo del Spooler Agent de SGI! Para mantener la calidad del código y la trazabilidad del sistema, seguimos reglas estrictas.

## 1. Flujo de Trabajo (GitFlow)
Tanto `main` como `develop` son ramas protegidas. Todo desarrollo se realiza en ramas auxiliares que nacen y mueren en `develop`:
* `feature/<nombre-tarea>`  - Nuevas funcionalidades.
* `bugfix/<nombre-error>`   - Corrección de fallos en desarrollo.
* `hotfix/<nombre-error>`   - Correcciones críticas urgentes directo a producción.

## 2. Estándar de Commits (Conventional Commits)
Todos los mensajes de commit deben seguir el estándar. Ejemplos:
* `feat: implementar lectura de hilos locales (Tarea #4)`
* `fix: corregir desbordamiento de memoria en el buffer (Tarea #9)`
* `chore: actualizar dependencias de seguridad`

## 3. Requisitos para el Merge (Definition of Done)
Antes de abrir un Pull Request hacia `develop`, asegúrate de ejecutar los tests locales para evitar commits innecesarios. Tu PR solo se integrará si:
1. El pipeline de CI está en verde (compila, pasa el linter y los tests automáticos)
2. Cuenta con la aprobación de al menos un revisor del equipo.
