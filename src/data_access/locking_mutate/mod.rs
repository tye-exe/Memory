//! Contains the [`locking_mutate`] macro.
//! See its documentation for more detail.

pub mod data_structures;

pub use data_structures::*;

// Used in docs
#[allow(unused_imports)]
use crate::data_access::{Da, Oda};
use std::sync::{Arc, MutexGuard};

#[macro_export]
/**
Holds the lock on the internal data for [`Da`] or [`Oda`] structs whilst executing the given
closure on the internal values.

# Examples

```
   use cell_memory::{
       data_access::{Da, Oda},
       locking_mutate,
   };

   let name = "AXE".to_owned();

   // Any Da or Oda can be passed in any order.
   let game_speed = Da::new(1.25f32);
   let score = Da::new(5);
   let highscore = Oda::new(8);

   // If you have the closure as a separate variable then rustfmt can format it for you.
   // You also explicitly need to specify type annotations for the closure, excluding the
   // return value.
   let closure = |mut player_score: u64, highscore: Option<u64>, mut game_speed: f32| {
       // The values passed into & returned by this closure are the internal data.
       // Any changes made will be applied to the data inside Da/Oda structs.
       player_score += 1;
       game_speed = game_speed.max(player_score as f32 / 10f32);

       // It's recommended to have to main logic outside of this macro, due to the lock
       // being held for the entire execution of the given closure.
       if let Some(mut highscore) = highscore {
           if player_score > highscore {
               highscore += 1;
           }
       }

       // As with normal closures you can also capture & use variables in the parent scope.
       let _message = format!("{name} - Score: {player_score}");
       // println!("{}", _message);

       // The values have to be returned in the same order they were given as a tuple.
       (player_score, highscore, game_speed)

       // These WOULD NOT compile
       // (player_score, game_speed)
       // (player_score, game_speed, highscore)

       // Returning `()` also fails.
   };

   // The Da/Oda are comma separated, with a ';' after the last one before the closure.
   locking_mutate!(score, highscore, game_speed; closure);

   // Values modified as expected.
   assert_eq!(game_speed.copy_value(), 1.25);
   assert_eq!(score.copy_value(), 6);
   assert_eq!(highscore.copy_value().unwrap(), 8);

   // Any values captured by the closure can still be safely used after.
   assert_eq!(name, "AXE");
```
*/
macro_rules! locking_mutate {
    ($($data_access:ident), +; $func:expr) => {
        {
            // This macro allows for creating new identities within rust code.
            // This is used to create unique local variables during repetitions, otherwise
            // the given identities would have to be shadowed. Making the desired functionality
            // impossible.
            use paste::paste;
            // Contains newtypes for converting values, as this macro has to deal with both
            // the `Data Access (Da)` & `Optional Data Access (Oda)` structs. Which isn't
            // possible as differeing methods have to be used to produce the same outcome
            // for each struct.
            // The newtypes abstract away this behaviour, as the methods used can't be modified
            // at compile time by this macro.
            use $crate::data_access::locking_mutate::data_structures::*;

            // Assigned each acquired mutex lock to unique local variables.
            // TODO: resolve possible dead-lock
            let ($(paste!{mut [<$data_access _lock>]}, )+) = ($($crate::data_access::locking_mutate::Lock::lock(&$data_access),)+);

            // Executes the given function/closure.
            let ($(paste!{[<$data_access _modified>]}, )+) = $func($(
                {
                    // Clones the value out of the `Arc` as the type isn't guaranteed to
                    // implement `Copy`
                    paste!{[<$data_access _lock>]}.ooa()
                },
            )+);

            // Replaces the internal values with returned values from the function/closure.
            $(
                *paste!{[<$data_access _lock>]} = {
                    // Split into separate lines to aid in legibility.
                    let value = paste!{[<$data_access _modified>]};
                    // See above comments for `data_structures` use expression.
                    let value = Wrapper::from(value);
                    value.into()
                };
            )+
        }
    };
}

/// Provides solitary access to data via a [`MutexGuard`].
pub trait Lock<Value> {
    /// The value contained within the returned [`MutexGuard`].
    type Returns;

    /// Returns a [`MutexGuard`] to the underlying data represented by this struct.
    fn lock(&self) -> MutexGuard<'_, Self::Returns>;
}

impl<Value> Lock<Value> for crate::data_access::Oda<Value>
where
    Value: 'static,
{
    type Returns = Option<Arc<Value>>;

    fn lock(&self) -> MutexGuard<'_, Self::Returns> {
        self.current_ref.lock().unwrap()
    }
}

impl<Value> Lock<Value> for crate::data_access::Da<Value>
where
    Value: 'static,
{
    type Returns = Arc<Value>;

    fn lock(&self) -> MutexGuard<'_, Self::Returns> {
        self.current_ref.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        data_access::{Da, Oda},
        test_data::Data,
    };

    #[test]
    fn da_lock() {
        let da_one = Da::new(Data::new(1));
        let da_two = Da::new(Data::new(2));

        locking_mutate!(da_one, da_two; |one: Data, two: Data| {
            (one + Data::new(1), two + Data::new(1))
        });

        assert_eq!(*da_one.get(), 2.into());
        assert_eq!(*da_two.get(), 3.into());
    }

    #[test]
    fn oda_lock() {
        let oda_one = Oda::new(Data::new(1));
        let oda_two = Oda::new(Data::new(2));

        locking_mutate!(oda_one, oda_two; |one: Option<Data>, two: Option<Data>| {
            (one.map(|value| value + 1), two.map(|value| value + 1))
        });

        assert_eq!(*oda_one.get().unwrap(), 2.into());
        assert_eq!(*oda_two.get().unwrap(), 3.into());
    }
}
