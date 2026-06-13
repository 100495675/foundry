#!/usr/bin/env bash

# Salir inmediatamente si un comando falla
set -e

echo "==============================================================================="
echo "FOUNDRY WORKSPACE - ESTRUCTURA DEL ÁRBOL"
echo "==============================================================================="

# Generar el árbol estructural ignorando binarios, cachés y metadatos de git/target
if command -v tree &> /dev/null; then
    tree -I 'target|.git|Cargo.lock|*.matrix|*.bin|debug|release'
else
    # Fallback elegante si 'tree' no está instalado en el sistema
    find . -not -path '*/.*' -not -path './target*' -not -path './.git*' | sort | sed 's/[^ corners\/ customs\-]/|  /g'
fi

echo -e "\n==============================================================================="
echo "FOUNDRY WORKSPACE - CONTENIDO ARCHIVO A ARCHIVO"
echo "==============================================================================="

# Buscar archivos relevantes (Cargo.toml y archivos de código de Rust .rs)
# Excluyendo estrictamente target, .git y archivos autogenerados de datos
find . -type f \( -name "Cargo.toml" -o -name "*.rs" \) \
    -not -path "*/target/*" \
    -not -path "*/.git/*" \
    -not -name "Cargo.lock" | sort | while read -r archivo; do

    echo "-------------------------------------------------------------------------------"
    echo "ARCHIVO: $archivo"
    echo "-------------------------------------------------------------------------------"
    
    # Imprimir el contenido del archivo con líneas numeradas para mejor lectura técnica
    cat -n "$archivo"
    echo -e "\n"
done

echo "==============================================================================="
echo "FIN DEL VOLCADO"
echo "==============================================================================="