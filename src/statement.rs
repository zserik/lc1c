use std::{fmt, str};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Argument {
    Absolute(u16),
    IdConst(u16),
    Label(String),
    Placeholder,
}

impl Argument {
    pub fn get_type(&self) -> (char, &'static str) {
        use Argument::*;
        match self {
            Absolute(_) => ('@', "absolute"),
            IdConst(_) => ('$', "ind.const"),
            Label(_) => (':', "label"),
            Placeholder => ('_', "place.holder"),
        }
    }

    pub fn take(&mut self) -> Self {
        std::mem::replace(self, Argument::Placeholder)
    }
}

impl fmt::Display for Argument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Argument::*;
        match self {
            Absolute(x) => write!(f, "@{}", x),
            IdConst(x) => write!(f, "${}", x),
            Label(ref x) => write!(f, "{}", x),
            Placeholder => write!(f, "@_"),
        }
    }
}

impl str::FromStr for Argument {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Argument::*;
        // match on first char
        Ok(match s.chars().next().unwrap() {
            '@' => Absolute(s.split_at(1).1.parse()?),
            '$' => IdConst(s.split_at(1).1.parse()?),
            _ => Label(s.to_string()),
        })
    }
}

pub trait StatementInvocBackend {
    type DefCode;
    type Label;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StatementInvocBase<T: StatementInvocBackend> {
    LDA(T),
    LDB(T),
    MOV(T),
    MAB,
    ADD,
    SUB,
    AND,
    NOT,

    JMP(T),
    JPS(T),
    JPO(T),
    CAL(T),
    RET,
    RRA(T),
    RLA(T),
    HLT,

    NOP,
    DEF(T::DefCode),
    Label(T::Label),
}

impl StatementInvocBackend for Argument {
    type DefCode = u16;
    type Label = String;
}

pub type StatementInvoc = StatementInvocBase<Argument>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Statement {
    pub invoc: StatementInvoc,
    pub optimizable: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Command {
    pub mnemonic: &'static str,
    pub is_real: bool,
    pub has_arg: bool,
}

impl StatementInvoc {
    pub fn into_statement(self, optimizable: bool) -> Statement {
        Statement {
            invoc: self,
            optimizable,
        }
    }
}

impl<T: StatementInvocBackend> StatementInvocBase<T> {
    fn map_or_fail<U, E, Fn, DFn, LFn, EFn>(
        self,
        f: Fn,
        df: DFn,
        lf: LFn,
        ef: EFn,
    ) -> Result<StatementInvocBase<U>, E>
    where
        U: StatementInvocBackend,
        Fn: FnOnce(T) -> Result<U, E>,
        DFn: FnOnce(T::DefCode) -> Result<U::DefCode, E>,
        LFn: FnOnce(T::Label) -> Result<U::Label, E>,
        EFn: FnOnce() -> Result<(), E>,
    {
        use StatementInvocBase::*;
        Ok(match self {
            LDA(x) => LDA(f(x)?),
            LDB(x) => LDB(f(x)?),
            MOV(x) => MOV(f(x)?),
            MAB => {
                ef()?;
                MAB
            }
            ADD => {
                ef()?;
                ADD
            }
            SUB => {
                ef()?;
                SUB
            }
            AND => {
                ef()?;
                AND
            }
            NOT => {
                ef()?;
                NOT
            }

            JMP(x) => JMP(f(x)?),
            JPS(x) => JPS(f(x)?),
            JPO(x) => JPO(f(x)?),
            CAL(x) => CAL(f(x)?),
            RET => {
                ef()?;
                RET
            }
            RRA(x) => RRA(f(x)?),
            RLA(x) => RLA(f(x)?),
            HLT => {
                ef()?;
                HLT
            }
            NOP => {
                ef()?;
                NOP
            }

            DEF(x) => DEF(df(x)?),
            Label(x) => Label(lf(x)?),
        })
    }

    pub fn arg(&self) -> Option<&T> {
        use StatementInvocBase::*;
        match self {
            LDA(ref x) | LDB(ref x) | MOV(ref x) | JMP(ref x) | JPS(ref x) | JPO(ref x)
            | CAL(ref x) | RRA(ref x) | RLA(ref x) => Some(x),
            _ => None,
        }
    }

    pub fn arg_mut(&mut self) -> Option<&mut T> {
        use StatementInvocBase::*;
        match self {
            LDA(ref mut x) | LDB(ref mut x) | MOV(ref mut x) | JMP(ref mut x) | JPS(ref mut x)
            | JPO(ref mut x) | CAL(ref mut x) | RRA(ref mut x) | RLA(ref mut x) => Some(x),
            _ => None,
        }
    }

    /// get_cmd -> (cmdcode, cmd2str, is_real, has_arg)
    pub fn get_cmd(&self) -> Command {
        macro_rules! cmd {
            (($cmd:ident), $is_real:expr) => { cmd!(@ $cmd, $is_real, true) };
            ($cmd:ident, $is_real:expr)   => { cmd!(@ $cmd, $is_real, false) };
            (@ $cmd:ident, $is_real:expr, $has_arg:expr) => {
                Command {
                    mnemonic: stringify!($cmd),
                    is_real: $is_real,
                    has_arg: $has_arg,
                }
            };
            (@ ($cmd:ident)) => { $cmd(_) };
            (@ $cmd:ident) => { $cmd };
        }
        macro_rules! cmds {
            ({$($tt_real:tt),+,}, {$($tt_virt:tt),+,}) => {
                match self {
                    $(cmd!(@ $tt_real) => cmd!($tt_real, true)),+,
                    $(cmd!(@ $tt_virt) => cmd!($tt_virt, false)),+,
                }
            }
        }
        use StatementInvocBase::*;
        cmds!(
            {
                (LDA), (LDB), (MOV), MAB,
                ADD, SUB, AND, NOT,
                (JMP), (JPS), (JPO), (CAL),
                RET, (RRA), (RLA),
                HLT, NOP,
            },
            {
                (DEF), (Label),
            }
        )
    }

    pub fn cmd2str(&self) -> &'static str {
        self.get_cmd().mnemonic
    }

    pub fn is_cmd_real(&self) -> bool {
        self.get_cmd().is_real
    }

    pub fn has_arg(self) -> bool {
        self.get_cmd().has_arg
    }

    pub fn take(&mut self) -> Self {
        std::mem::replace(self, StatementInvocBase::NOP)
    }
}

impl fmt::Display for StatementInvoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatementInvoc::DEF(x) => write!(f, "DEF {}", x),
            StatementInvoc::Label(ref x) => write!(f, "{}:", x),
            _ => {
                write!(f, "{}", self.cmd2str())?;
                if let Some(x) = self.arg() {
                    write!(f, " {}", x)
                } else {
                    Ok(())
                }
            }
        }
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ParseStatementError {
    #[error("statement is invalid because it's too short")]
    TooShort,
    #[error("expected no argument, found one")]
    UnexpectedArgument,
    #[error("expected one argument, found none")]
    ArgumentNotFound,
    #[error("argument is invalid")]
    InvalidArgument,
    #[error("statement consists of too many (whitespace-separated) tokens (expected at most 2, got {0})")]
    TooManyTokens(usize),
    #[error("got unknown command")]
    UnknownCommand,
    #[error("got forbidden inline label")]
    InlineLabel,

    #[error("parsing argument failed: {0}")]
    Integer(#[from] std::num::ParseIntError),
}

struct ParserWoArg;

impl StatementInvocBackend for ParserWoArg {
    type DefCode = ParserWoArg;
    type Label = String;
}

impl str::FromStr for StatementInvoc {
    type Err = ParseStatementError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ParseStatementError::*;
        use StatementInvocBase::*;

        if s.len() < 2 {
            Err(TooShort)
        } else if s.ends_with(':') {
            // got label
            Ok(Label(s.split_at(s.len() - 1).0.to_string()))
        } else {
            let (cmd, arg) = {
                let parts: Vec<&str> = s.split_whitespace().collect();
                let arg = match parts.len() {
                    0 => Err(TooShort),
                    1 => Ok(None),
                    2 => Ok(Some(parts[1])),
                    n => Err(if parts[0].ends_with(':') {
                        InlineLabel
                    } else {
                        TooManyTokens(n)
                    }),
                }?;
                (parts[0], arg)
            };
            let arg_ok = || arg.ok_or(ArgumentNotFound);
            Ok(match cmd.to_uppercase().as_str() {
                "LDA" => LDA(ParserWoArg),
                "LDB" => LDB(ParserWoArg),
                "MOV" => MOV(ParserWoArg),
                "MAB" => MAB,
                "ADD" => ADD,
                "SUB" => SUB,
                "AND" => AND,
                "NOT" => NOT,

                "JMP" => JMP(ParserWoArg),
                "JPS" => JPS(ParserWoArg),
                "JPO" => JPO(ParserWoArg),
                "CAL" => CAL(ParserWoArg),
                "RET" => RET,
                "RRA" => RRA(ParserWoArg),
                "RLA" => RLA(ParserWoArg),
                "HLT" => HLT,
                "NOP" => NOP,

                "DEF" => DEF(ParserWoArg),
                _ => {
                    return Err(if cmd.find(':').is_some() {
                        InlineLabel
                    } else {
                        UnknownCommand
                    })
                }
            }
            .map_or_fail(
                |_| Ok(arg_ok()?.parse::<Argument>()?),
                |_| Ok(arg_ok()?.parse::<u16>()?),
                Ok,
                || {
                    if arg.is_some() {
                        Err(UnexpectedArgument)
                    } else {
                        Ok(())
                    }
                },
            )?)
        }
    }
}
