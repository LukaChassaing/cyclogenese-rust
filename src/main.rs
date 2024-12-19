use std::f64::consts::PI;
use std::error::Error;
use std::fmt;

/// Constantes physiques regroupées dans une structure pour une meilleure organisation
#[derive(Debug, Clone, Copy)]
pub struct PhysicalConstants {
    earth_omega: f64,      // Vitesse de rotation de la Terre (rad/s)
    gravity: f64,          // Accélération gravitationnelle (m/s²)
    base_temp: f64,       // Température de référence (K)
}

impl Default for PhysicalConstants {
    fn default() -> Self {
        Self {
            earth_omega: 7.2921e-5,
            gravity: 9.81,
            base_temp: 288.15,
        }
    }
}

/// Types d'erreurs personnalisés
#[derive(Debug)]
pub enum MeteoError {
    InvalidLatitude(f64),
    InvalidPressure(f64),
    InvalidTemperature(f64),
    InvalidAltitude(f64),
}

impl fmt::Display for MeteoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MeteoError::InvalidLatitude(lat) => write!(f, "Latitude invalide: {}°", lat),
            MeteoError::InvalidPressure(p) => write!(f, "Pression invalide: {} hPa", p),
            MeteoError::InvalidTemperature(t) => write!(f, "Température invalide: {} K", t),
            MeteoError::InvalidAltitude(a) => write!(f, "Altitude invalide: {} m", a),
        }
    }
}

impl Error for MeteoError {}

/// Résultats du développement de la perturbation
#[derive(Debug, Clone)]
pub struct DevelopmentResult {
    vertical_velocity: f64,
    relative_vorticity: f64,
    hour: u32,
}

impl DevelopmentResult {
    /// Convertit les résultats en format lisible
    pub fn to_string_formatted(&self) -> String {
        format!("{:4} | {:20.2} | {:20.2}",
            self.hour,
            self.vertical_velocity * 100.0,  // Conversion en cm/s
            self.relative_vorticity * 1e5    // Conversion en 10⁻⁵ s⁻¹
        )
    }
}

/// Position géographique et conditions atmosphériques
#[derive(Debug, Clone)]
pub struct Position {
    latitude: f64,
    altitude: f64,
    pressure: f64,
}

impl Position {
    /// Crée une nouvelle position avec validation
    pub fn new(latitude: f64, altitude: f64, pressure: f64) -> Result<Self, MeteoError> {
        if !(-90.0..=90.0).contains(&latitude) {
            return Err(MeteoError::InvalidLatitude(latitude));
        }
        if altitude < -400.0 || altitude > 20000.0 {
            return Err(MeteoError::InvalidAltitude(altitude));
        }
        if pressure < 100.0 || pressure > 1100.0 {
            return Err(MeteoError::InvalidPressure(pressure));
        }

        Ok(Self {
            latitude,
            altitude,
            pressure,
        })
    }
}

/// Anomalie thermique
#[derive(Debug)]
pub struct ThermalAnomaly {
    temperature_delta: f64,
    position: Position,
    is_cyclonic: bool,
    intensity: f64,
    constants: PhysicalConstants,
}

impl ThermalAnomaly {
    /// Crée une nouvelle anomalie thermique
    pub fn new(
        temperature_delta: f64,
        position: Position,
        constants: PhysicalConstants,
    ) -> Result<Self, MeteoError> {
        if !(-50.0..=50.0).contains(&temperature_delta) {
            return Err(MeteoError::InvalidTemperature(temperature_delta));
        }

        Ok(Self {
            temperature_delta,
            position,
            is_cyclonic: temperature_delta > 0.0,
            intensity: 1.0,
            constants,
        })
    }

    fn compute_coriolis_force(&self) -> f64 {
        self.constants.earth_omega * (self.position.latitude * PI / 180.0).sin()
    }

    fn compute_relative_vorticity(&self, thermal_wind: f64) -> f64 {
        const RADIUS: f64 = 5.0e5;  // 500 km
        const AMPLIFICATION: f64 = 1.0e3;
        
        let base_vorticity = thermal_wind / RADIUS;
        let altitude_factor = if self.position.pressure < 500.0 { 2.0 } else { 1.0 };
        
        if self.is_cyclonic {
            base_vorticity * self.intensity * altitude_factor * AMPLIFICATION
        } else {
            -base_vorticity * self.intensity * altitude_factor * AMPLIFICATION
        }
    }

    fn develop_baroclinic_perturbation(&mut self, hour: u32) -> DevelopmentResult {
        // Mise à jour de l'intensité
        self.intensity = 1.0 + (hour as f64 / 12.0);
        
        let coriolis = self.compute_coriolis_force();
        
        // Calcul du vent thermique
        let base_wind = self.temperature_delta / self.constants.base_temp * 
                       self.constants.gravity * 1000.0;
        let thermal_wind = if self.is_cyclonic {
            base_wind * coriolis
        } else {
            -base_wind * coriolis
        };

        // Calcul de la vitesse verticale
        let pressure_factor = (1000.0 / self.position.pressure).sqrt();
        let altitude_factor = (-self.position.altitude / 8000.0).exp();
        
        let vertical_velocity = if self.position.pressure > 500.0 {
            thermal_wind * 0.1 * pressure_factor * altitude_factor
        } else {
            -thermal_wind * 0.1 * pressure_factor * altitude_factor
        } * self.intensity;

        let relative_vorticity = self.compute_relative_vorticity(thermal_wind);

        DevelopmentResult {
            vertical_velocity,
            relative_vorticity,
            hour,
        }
    }
}

/// Structure principale pour la simulation de cyclogénèse
pub struct BaroclinicCyclogenesis {
    surface_anomaly: ThermalAnomaly,
    altitude_anomaly: ThermalAnomaly,
    baroclinic_zone: bool,
}

impl BaroclinicCyclogenesis {
    /// Crée une nouvelle instance de simulation
    pub fn new(
        surface_temp: f64,
        altitude_temp: f64,
        latitude: f64,
    ) -> Result<Self, MeteoError> {
        let constants = PhysicalConstants::default();
        
        let surface_position = Position::new(latitude, 0.0, 1013.0)?;
        let altitude_position = Position::new(latitude, 5000.0, 500.0)?;
        
        let surface_anomaly = ThermalAnomaly::new(
            surface_temp,
            surface_position,
            constants,
        )?;
        
        let altitude_anomaly = ThermalAnomaly::new(
            altitude_temp,
            altitude_position,
            constants,
        )?;

        Ok(Self {
            surface_anomaly,
            altitude_anomaly,
            baroclinic_zone: true,
        })
    }

    /// Simule l'interaction entre les anomalies
    pub fn simulate_interaction(&mut self, time_steps: u32) -> Vec<DevelopmentResult> {
        let mut results = Vec::with_capacity(time_steps as usize);
        
        for hour in 0..time_steps {
            let surface_result = self.surface_anomaly.develop_baroclinic_perturbation(hour);
            let altitude_result = self.altitude_anomaly.develop_baroclinic_perturbation(hour);
            
            let interaction_factor = if self.baroclinic_zone {
                1.5 * (1.0 + hour as f64 / 24.0)
            } else {
                1.0
            };
            
            results.push(DevelopmentResult {
                vertical_velocity: (surface_result.vertical_velocity + 
                                  altitude_result.vertical_velocity) * interaction_factor,
                relative_vorticity: (surface_result.relative_vorticity + 
                                   altitude_result.relative_vorticity) * interaction_factor,
                hour,
            });
        }
        
        results
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let latitudes = vec![30.0, 45.0, 60.0];
    
    println!("SIMULATION DE CYCLOGÉNÈSE BAROCLINE");
    println!("====================================\n");
    
    for latitude in latitudes {
        println!("\nSimulation à {}°N :", latitude);
        println!("Heure | Vitesse verticale (cm/s) | Tourbillon relatif (10⁻⁵ s⁻¹)");
        println!("------|----------------------|----------------------");
        
        let mut cyclogenesis = BaroclinicCyclogenesis::new(5.0, -8.0, latitude)?;
        let results = cyclogenesis.simulate_interaction(24);
        
        for result in results {
            println!("{}", result.to_string_formatted());
        }
    }

    Ok(())
}