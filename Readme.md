# Simulation de Cyclogenèse Barocline en Rust

Ce programme implémente une simulation de cyclogenèse barocline basée sur les concepts fondamentaux de la météorologie dynamique des moyennes latitudes.

## Contexte Théorique

### Structure Barocline de l'Atmosphère

La simulation se base sur la structure barocline de l'atmosphère dans les latitudes tempérées (30°N-70°N), où l'écoulement atmosphérique est déterminé par trois forces principales :
- Force de Coriolis
- Force de Pression
- Force de Gravité

Ces forces, combinées au chauffage différentiel pôle-équateur, conduisent à une structure atmosphérique barocline caractérisée par une inclinaison des surfaces isobares par rapport aux surfaces isothermes.

### Cyclogenèse par Interaction Barocline

Le processus de cyclogenèse est modélisé à travers plusieurs composantes clés :

1. **Anomalies Thermiques** :
   - En surface : anomalie chaude (cyclonique)
   - En altitude : anomalie froide (anticyclonique)
   - Ces anomalies sont représentées par la structure `ThermalAnomaly`

2. **Développement Vertical** :
   - Ascendances à l'avant de l'anomalie
   - Subsidences à l'arrière
   - Modélisé par le calcul des vitesses verticales dans `develop_baroclinic_perturbation`

3. **Tourbillon Relatif** :
   - Calcul basé sur le vent thermique et la force de Coriolis
   - Intensification progressive avec le développement de la perturbation
   - Implémenté dans `compute_relative_vorticity`

## Structure du Code

### Constantes Physiques
```rust
pub struct PhysicalConstants {
    earth_omega: f64,      // Vitesse de rotation de la Terre (rad/s)
    gravity: f64,          // Accélération gravitationnelle (m/s²)
    base_temp: f64,        // Température de référence (K)
}
```

### Paramètres Atmosphériques
```rust
pub struct Position {
    latitude: f64,    // Position en degrés Nord
    altitude: f64,    // Altitude en mètres
    pressure: f64,    // Pression en hPa
}
```

### Simulation de l'Interaction Barocline

La classe `BaroclinicCyclogenesis` simule l'interaction entre :
- Une anomalie de surface (chaude)
- Une anomalie d'altitude (froide)
- L'interaction est renforcée dans la zone barocline

## Paramètres de Simulation

- **Latitudes** : 30°N à 60°N
- **Niveaux de pression** :
  - Surface : 1013 hPa
  - Altitude : 500 hPa
- **Anomalies de température** :
  - Surface : +5°K
  - Altitude : -8°K

## Résultats

La simulation produit deux paramètres principaux :

1. **Vitesses Verticales** :
   - Unité : cm/s
   - Augmentent avec la latitude
   - S'intensifient avec le temps

2. **Tourbillon Relatif** :
   - Unité : 10⁻⁵ s⁻¹
   - Plus fort aux latitudes élevées
   - Développement progressif

## Utilisation

```rust
// Création d'une nouvelle simulation
let mut cyclogenesis = BaroclinicCyclogenesis::new(5.0, -8.0, 45.0)?;

// Simulation sur 24 heures
let results = cyclogenesis.simulate_interaction(24);

// Affichage des résultats
for result in results {
    println!("{}", result.to_string_formatted());
}
```

## Validation et Gestion des Erreurs

Le code inclut une validation complète des paramètres d'entrée :
- Latitudes valides : -90° à +90°
- Pressions valides : 100 à 1100 hPa
- Altitudes valides : -400 à 20000 m
- Anomalies de température : -50 à +50 K

Les erreurs sont gérées via un type personnalisé `MeteoError`.

## Notes

j'ai conçu ce programme comme un mémo et une mise en pratique des notions suivantes :
1. Les mécanismes fondamentaux de la cyclogenèse barocline
2. L'interaction entre les différentes couches atmosphériques
3. L'influence de la latitude sur le développement des dépressions
4. L'importance de la structure thermique dans la dynamique atmosphérique

Toutes les informations sont disponibles dans le Tome 1 de `Concepts et Méthodes pour le météorologiste` de *Christophe Calas*.