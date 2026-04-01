/**
 * Typedefs for tof relevant code.
 *
 * Rationale: Make sure that numeric types
 *            have the same size on different
 *            systems, which is relevant for 
 *            (de)serialization.
 *
 * Bonus: Match rust syntax a bit more closely,
 *        so it is easier to compare the C++
 *        to its Rust counterpart
 *        Always remember! The rust library is 
 *        the gold standard since that is what 
 *        is run on the TOF computer
 *
 */
#ifndef GAPSTOFTYPEDEFS_H_INCLUDED
#define GAPSTOFTYPEDEFS_H_INCLUDED

#include <vector>
#include <map>
#include <cstddef>
#include <string>
#include <cstdint>
#include <optional>
#include <stdexcept>
#include <utility>

typedef uint8_t   u8;
typedef uint16_t  u16;
typedef uint32_t  u32;
typedef uint64_t  u64;
typedef int8_t    i8;
typedef int16_t   i16;
typedef int32_t   i32;
typedef int64_t   i64;
typedef size_t    usize;
///FIXME - get the correct type for float
typedef float     f32;
///FIXME - get the correct type for double
typedef double    f64;
typedef std::string String;

/// Define vectors the same as in Rust
template <typename T>
using Vec = std::vector<T>;

/// Define std::map the same as HashMap in Rust
template <typename T, typename U>
using HashMap = std::map<T,U>;

//-----------------------------------------------------

template <typename T>
struct Option : public std::optional<T> {
    using std::optional<T>::optional; // Inherit all std::optional constructors

    // --- Rust-style State Checks ---

    [[nodiscard]] constexpr bool is_some() const noexcept { 
        return this->has_value(); 
    }

    [[nodiscard]] constexpr bool is_none() const noexcept { 
        return !this->has_value(); 
    }

    // --- Rust-style Unwrapping ---

    // For lvalues (variables): returns a reference
    constexpr T& unwrap() & {
        if (is_none()) {
            throw std::runtime_error("called `Option::unwrap()` on a `None` value");
        }
        return this->value();
    }

    // For rvalues (temporaries): moves the value out efficiently
    constexpr T&& unwrap() && {
        if (is_none()) {
            throw std::runtime_error("called `Option::unwrap()` on a `None` value");
        }
        return std::move(this->value());
    }
};

// Helpers for syntax
inline constexpr std::nullopt_t None = std::nullopt;

template <typename T>
constexpr Option<std::decay_t<T>> Some(T&& value) {
    return Option<std::decay_t<T>>(std::forward<T>(value));
}

//-----------------------------------------------------

typedef HashMap<u8,HashMap<u8,HashMap<u8, std::pair<u8,u8>>>> LtbRBMap;

typedef Vec<u8> bytestream;
#endif
