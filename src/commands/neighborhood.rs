//! `neighborhood` — la ventana numérica alrededor de un episodio.
//!
//! Es un comando de **solo lectura**: no escribe el tracker ni toca el material.
//! La tabla es exactamente la de `list` (misma cabecera, separador, formato de
//! fila, marca `*` y leyenda), con la fila de la referencia marcada con `► `.
//!
//! La ventana se calcula **por número**, nunca por fecha: los episodios sin
//! número no tienen posición numérica y quedan fuera. El episodio de referencia
//! debe tener número; si no lo tiene, el comando falla (no hay vecindad que
//! calcular).

use crate::commands::{Ctx, Rendered, resolve_episode, show_episodes};
use crate::error::PodtrackError;
use crate::json;
use crate::model::Episode;

/// Ventana numérica alrededor de un episodio de referencia.
pub struct Neighborhood<'a> {
    /// Vecinos con número estrictamente menor, ascendentes (el último es el inmediato inferior).
    pub before: Vec<&'a Episode>,
    /// El episodio de referencia: la única fila marcada.
    pub episode: &'a Episode,
    /// Vecinos con número estrictamente mayor, ascendentes (el primero es el inmediato superior).
    pub after: Vec<&'a Episode>,
}

impl<'a> Neighborhood<'a> {
    /// Filas de la tabla en el orden en que se muestran.
    #[must_use]
    pub fn rows(&self) -> Vec<&'a Episode> {
        let mut rows: Vec<&'a Episode> = Vec::with_capacity(
            self.before
                .len()
                .saturating_add(1)
                .saturating_add(self.after.len()),
        );
        rows.extend(self.before.iter().copied());
        rows.push(self.episode);
        rows.extend(self.after.iter().copied());
        rows
    }
}

/// Selecciona la ventana numérica alrededor de `reference`.
///
/// La ventana contiene **exactamente** los vecinos con número *estrictamente*
/// menor que el de la referencia (los más cercanos, hasta `before`), la
/// referencia, y los de número *estrictamente* mayor (los más cercanos, hasta
/// `after`); siempre en orden ascendente de número (estable: con números
/// repetidos se conserva el orden del tracker).
///
/// Los episodios sin número no tienen posición numérica y quedan fuera. Los
/// episodios con el **mismo número** que la referencia tampoco entran: no son
/// anteriores ni siguientes, así que el marcador aparece **una sola vez** y
/// `before`/`after` se cuentan por episodio (los duplicados no consumen
/// presupuesto).
///
/// La referencia se localiza **por identidad** (posición en el tracker), no por
/// número: es la misma regla que en el resto del proyecto. Si la referencia no
/// tiene número, o no está en `episodes`, devuelve la ventana con la referencia
/// y las dos listas vacías (el selector es total y nunca entra en pánico).
#[must_use]
pub fn select_neighborhood<'a>(
    episodes: &'a [Episode],
    reference: &'a Episode,
    before: usize,
    after: usize,
) -> Neighborhood<'a> {
    let sola = || Neighborhood {
        before: Vec::new(),
        episode: reference,
        after: Vec::new(),
    };

    let Some(number) = reference.number else {
        return sola();
    };

    let mut numbered: Vec<&Episode> = episodes
        .iter()
        .filter(|episode| episode.number.is_some())
        .collect();
    numbered.sort_by_key(|episode| episode.number);

    let Some(index) = numbered
        .iter()
        .position(|episode| std::ptr::eq(*episode, reference))
    else {
        return sola();
    };

    let anteriores: Vec<&Episode> = numbered
        .get(..index)
        .unwrap_or_default()
        .iter()
        .copied()
        .filter(|episode| episode.number.is_some_and(|n| n < number))
        .collect();
    let start = anteriores.len().saturating_sub(before);
    let before_vec = anteriores
        .get(start..)
        .map(<[&Episode]>::to_vec)
        .unwrap_or_default();

    let after_vec: Vec<&Episode> = numbered
        .get(index.saturating_add(1)..)
        .unwrap_or_default()
        .iter()
        .copied()
        .filter(|episode| episode.number.is_some_and(|n| n > number))
        .take(after)
        .collect();

    Neighborhood {
        before: before_vec,
        episode: reference,
        after: after_vec,
    }
}

/// Ejecuta `neighborhood`.
///
/// # Errors
///
/// - [`PodtrackError::EpisodeNotFound`] si `target` no resuelve a un episodio.
/// - [`PodtrackError::NeighborhoodWithoutNumber`] si el episodio resuelto no
///   tiene número (una vecindad numérica necesita uno).
pub fn run(
    ctx: &Ctx<'_>,
    target: &str,
    before: usize,
    after: usize,
    json_mode: bool,
) -> Result<Rendered, PodtrackError> {
    let reference =
        resolve_episode(ctx.tracker, target).ok_or_else(|| PodtrackError::EpisodeNotFound {
            target: target.to_string(),
        })?;

    let Some(number) = reference.number else {
        return Err(PodtrackError::NeighborhoodWithoutNumber {
            label: reference.slug_str().unwrap_or(target).to_string(),
        });
    };

    let ventana = select_neighborhood(&ctx.tracker.episodes, reference, before, after);

    if json_mode {
        return Ok(Rendered::stdout(json::neighborhood_document(
            ctx,
            ventana.episode,
            &ventana.before,
            &ventana.after,
        )));
    }

    let mut out = String::new();
    let mut err = String::new();
    out.push_str(&format!(
        "\n  Vecindad de #{number} (antes: {}, después: {}):\n",
        ventana.before.len(),
        ventana.after.len()
    ));
    show_episodes(
        &ventana.rows(),
        true,
        Some(ventana.episode),
        &mut out,
        &mut err,
    );
    Ok(Rendered::new(out, err))
}

#[cfg(test)]
mod tests {
    use super::select_neighborhood;
    use crate::model::{Date, Episode, Status};

    fn ep(number: Option<u32>, date: &str) -> Episode {
        Episode {
            number,
            slug: None,
            season: 8,
            date: Date::parse(date),
            title: match number {
                Some(n) => format!("Episodio {n}"),
                None => "Sin número".to_string(),
            },
            status: Status::Published,
        }
    }

    fn numeros(window: &[&Episode]) -> Vec<Option<u32>> {
        window.iter().map(|episode| episode.number).collect()
    }

    /// Referencia por identidad: el primer episodio con ese número.
    fn referencia(episodes: &[Episode], number: u32) -> &Episode {
        episodes
            .iter()
            .find(|episode| episode.number == Some(number))
            .expect("episodio de referencia")
    }

    #[test]
    fn ventana_centrada_en_una_parrilla_completa() {
        // Un episodio sin número en medio: no tiene posición numérica y no entra.
        let mut episodes: Vec<Episode> = (674..=682).map(|n| ep(Some(n), "2026-09-01")).collect();
        episodes.insert(4, ep(None, "2026-09-02"));
        let reference = referencia(&episodes, 678);

        let ventana = select_neighborhood(&episodes, reference, 2, 2);

        assert_eq!(
            numeros(&ventana.rows()),
            vec![Some(676), Some(677), Some(678), Some(679), Some(680)]
        );
    }

    #[test]
    fn huecos_devuelven_los_existentes_mas_cercanos() {
        let episodes = vec![
            ep(Some(670), "2026-09-01"),
            ep(Some(672), "2026-09-03"),
            ep(Some(678), "2026-09-09"),
            ep(Some(681), "2026-09-12"),
            ep(Some(690), "2026-09-21"),
        ];
        let reference = referencia(&episodes, 678);

        let ventana = select_neighborhood(&episodes, reference, 4, 4);

        assert_eq!(
            numeros(&ventana.rows()),
            vec![Some(670), Some(672), Some(678), Some(681), Some(690)]
        );
    }

    #[test]
    fn extremo_devuelve_menos_vecinos_sin_error() {
        let episodes: Vec<Episode> = (100..=105).map(|n| ep(Some(n), "2026-09-01")).collect();
        let reference = referencia(&episodes, 101);

        let ventana = select_neighborhood(&episodes, reference, 4, 4);

        assert_eq!(
            numeros(&ventana.rows()),
            vec![
                Some(100),
                Some(101),
                Some(102),
                Some(103),
                Some(104),
                Some(105)
            ]
        );
    }

    #[test]
    fn before_y_after_cero_dejan_solo_lo_pedido() {
        let episodes: Vec<Episode> = (674..=682).map(|n| ep(Some(n), "2026-09-01")).collect();
        let reference = referencia(&episodes, 678);

        let ventana = select_neighborhood(&episodes, reference, 0, 0);

        assert_eq!(numeros(&ventana.rows()), vec![Some(678)]);

        // Referencia ausente del tracker: ventana sin vecinos, referencia intacta.
        let fuera = ep(Some(999), "2026-09-01");
        let ventana = select_neighborhood(&episodes, &fuera, 4, 4);
        assert!(
            ventana.before.is_empty(),
            "antes: {:?}",
            numeros(&ventana.before)
        );
        assert!(
            ventana.after.is_empty(),
            "después: {:?}",
            numeros(&ventana.after)
        );
        assert!(std::ptr::eq(ventana.episode, &fuera));

        // Referencia sin número: tampoco hay ventana.
        let sin_numero = ep(None, "2026-09-01");
        let ventana = select_neighborhood(&episodes, &sin_numero, 4, 4);
        assert!(ventana.before.is_empty() && ventana.after.is_empty());
        assert!(std::ptr::eq(ventana.episode, &sin_numero));
    }

    #[test]
    fn numeros_repetidos_dejan_fuera_a_los_duplicados() {
        // 10, 11, 11, 11, 12, 13 con la referencia en el segundo `11`.
        let episodes = vec![
            ep(Some(10), "2026-09-01"),
            ep(Some(11), "2026-09-02"),
            ep(Some(11), "2026-09-03"),
            ep(Some(11), "2026-09-04"),
            ep(Some(12), "2026-09-05"),
            ep(Some(13), "2026-09-06"),
        ];
        let reference = &episodes[2];

        let ventana = select_neighborhood(&episodes, reference, 1, 4);

        assert_eq!(numeros(&ventana.before), vec![Some(10)]);
        assert_eq!(numeros(&ventana.after), vec![Some(12), Some(13)]);
        assert!(std::ptr::eq(ventana.episode, reference));
        assert_eq!(
            numeros(&ventana.rows()),
            vec![Some(10), Some(11), Some(12), Some(13)]
        );
    }
}
