// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

using System;
using System.Collections.Generic;

namespace Serde
{
    public readonly struct Option<T> : IEquatable<Option<T>> where T : IEquatable<T>
    {
        public static Option<T> None => default;
        public static Option<T> Some(T value)
        {
            if (value == null) throw new ArgumentNullException(nameof(value));
            return new Option<T>(value);
        }

        readonly bool isSome;
        readonly T value;

        Option(T val)
        {
            isSome = val != null;
            value = val;
        }

        public bool IsSome(out T value)
        {
            value = this.value;
            return isSome;
        }

        public override bool Equals(object obj) => obj is Option<T> option && Equals(option);

        public bool Equals(Option<T> other)
        {
            if (isSome != other.isSome) return false;
            if (!isSome) return true;
            return value.Equals(other.value);
        }

        public override int GetHashCode()
        {
            var hashCode = -934799437;
            hashCode = hashCode * -1521134295 + isSome.GetHashCode();
            hashCode = hashCode * -1521134295 + EqualityComparer<T>.Default.GetHashCode(value);
            return hashCode;
        }

        public static bool operator ==(Option<T> left, Option<T> right) => Equals(left, right);

        public static bool operator !=(Option<T> left, Option<T> right) => !Equals(left, right);
    }

    public static class OptionExtensions
    {
        public static U Match<T, U>(this Option<T> option, Func<T, U> onIsSome, Func<U> onIsNone)
        where T : IEquatable<T> where U : IEquatable<U> =>
            option.IsSome(out var value) ? onIsSome(value) : onIsNone();

        public static Option<U> Bind<T, U>(this Option<T> option, Func<T, Option<U>> binder)
        where T : IEquatable<T> where U : IEquatable<U> =>
            option.Match(onIsSome: binder, onIsNone: () => Option<U>.None);

        public static Option<U> Map<T, U>(this Option<T> option, Func<T, U> mapper)
        where T : IEquatable<T> where U : IEquatable<U> =>
            option.Bind(value => Option<U>.Some(mapper(value)));

        public static Option<T> Filter<T>(this Option<T> option, Predicate<T> predicate)
        where T : IEquatable<T> =>
            option.Bind(value => predicate(value) ? option : Option<T>.None);

        public static T DefaultValue<T>(this Option<T> option, T defaultValue)
        where T : IEquatable<T> =>
            option.Match(onIsSome: value => value, onIsNone: () => defaultValue);
    }
}
