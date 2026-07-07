\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{jstarticle}[2024-03-18 Hanoi University of Science's Journal of Science and Technology Article]
\RequirePackage{expl3}
\RequirePackage{pgfopts}
\RequirePackage{graphicx}
\RequirePackage{xparse}
\RequirePackage[fleqn]{amsmath}
\RequirePackage{amssymb}

% https://stackoverflow.com/questions/53167913/how-to-place-the-figure-at-the-bottom-of-page-in-latex
\RequirePackage{stfloats}

% Requirements and technical notes >>>
% ============
% The paper is presented in a two-column format with a column spacing of 0.7 cm.  The paper size is A4, portrait. Page setups are with margins of 
% Top: 3 cm,
% Bottom: 2.5 cm,
% Left: 2.5 cm,
% Right: 2 cm;
% Header: 1.27 cm,
% Footer: 1.27 cm. 
% Text is written in Time New Roman font, size 10 point, normal, and justified.
% Space after paragraphs is 6 pt.
% The first line of each paragraph has an indentation of 0.5 cm.
% Except for the abstract being written in the font of Time New Roman, all other parts of the paper use the Times New Roman font

% Footnote IS A VERY HARD PROBLEM. 
% If we use \multicols, the footnote is correct, BUT, float will not work
%<<<

% Load base classes>>>
% =================
\PassOptionsToClass{\CurrentOption}{article}
\ExecuteOptions{a4paper, fleqn}
\LoadClass{article}
\providecommand{\pgfsyspdfmark}[3]{}
\RequirePackage{flushend}
\RequirePackage{flushend}


% Custom options
\providecommand{\jst@mathindent}{\mathindent}
\pgfkeys{
	/jstarticle/.cd,
	mathindent/.store in=\jst@mathindent,
}
\ProcessPgfOptions{/jstarticle}
\setlength{\mathindent}{\jst@mathindent}
%<<<

% Allow breaking multiline displays
% Elsevier also uses this, so it's
% Probably ok
\allowdisplaybreaks

% Font sizes >>>
% ==============
% The "pt" in words are different from the one from latex
% The word's pt refer to bp
% https://tex.stackexchange.com/questions/34024/setting-a-document-in-ms-word-12pt-12bp
% https://tex.stackexchange.com/questions/8260/what-are-the-various-units-ex-em-in-pt-bp-dd-pc-expressed-in-mm
% \setlength{\p@}{1bp}
% \RequirePackage{microtype}


% \ExplSyntaxOn
% \AddToHook{begindocument/before}{%
% 	\sys_if_engine_pdftex:TF {
% 		\RequirePackage{tgtermes}
% 		\RequirePackage{tgheros}
% 		\RequirePackage{libertinust1math}
% 	} {
% 		\RequirePackage{fontspec}
% 		%\RequirePackage[notext, not1]{stix}
% 		\RequirePackage{unicode-math}
% 		\setmainfont{Times New Roman}%[SizeFeatures={Size=10}]
% 		\setsansfont{Arial}%[SizeFeatures={Size=10}]
% 		\setmathfont{TeX Gyre Termes Math}
% 	}
% }

\ExplSyntaxOn
\AddToHook{begindocument/before}{%
    \sys_if_engine_pdftex:TF {
        \usepackage{times}      
        \usepackage{helvet}      
        \usepackage{courier}   
        \usepackage{mathptmx}    
        \usepackage{fontspec}
        \usepackage{unicode-math}
        \setmainfont{times.ttf}[
            Path = Fonts/,
            BoldFont = timesbd.ttf,
            ItalicFont = timesi.ttf,
            BoldItalicFont = timesbi.ttf,
            SizeFeatures={Size=10}
        ]
        \setsansfont{arial.ttf}[
            Path = Fonts/,
            BoldFont = arialbd.ttf,
            ItalicFont = ariali.ttf,
            BoldItalicFont = arialbi.ttf
        ]
        \setmathfont{TeX Gyre Termes Math}
    }
}


\ExplSyntaxOff
\def\titleFont{\fontsize{14pt}{14pt}\selectfont\bfseries}
\def\authorFont{\fontsize{12pt}{12pt}\selectfont\itshape\bfseries}
\def\affiliationFont{\fontsize{10pt}{10pt}\selectfont\itshape}
\def\abstractFont{\fontsize{9pt}{9pt}\selectfont\rmfamily\itshape}
\def\keywordFont{\fontsize{9pt}{9pt}\selectfont\rmfamily}
\def\headingAFont{\fontsize{10pt}{12pt}\selectfont\bfseries}
\def\headingBFont{\fontsize{10pt}{12pt}\selectfont\bfseries\itshape}
\def\headingCFont{\fontsize{10pt}{12pt}\selectfont\itshape}
\renewcommand{\footnotesize}{\fontsize{9pt}{11pt}\selectfont}
% <<<

% Make an alias for \@date to play nice with Expl3>>>
\makeatletter
\renewcommand{\today}{%
  \ifcase\month\or
    January\or February\or March\or April\or May\or June\or
    July\or August\or September\or October\or November\or December%
  \fi~ \number\year}
\def\insertdate{\today}
\makeatother
%<<<

% Mark start and end page>>>
% =======================
% \RequirePackage{lastpage}
\AddToHook{begindocument/end}{%
	\setcounter{page}{\pageoffsetnumber}%
}
\AddToHook{begindocument/end}{\label{JST::startpage}}
\AddToHook{enddocument}{\label{JST::endpage}}
\def\insertstartpage{\getpagerefnumber{JST::startpage}}
\def\insertendpage{\getpagerefnumber{JST::endpage}}
%<<<

% Helper functions>>>
% ================

% Pad the number to 3 zero digits>>>
% -------------------------------
\let\oldpageref\pageref
% Zeropad to 3 characters
\ExplSyntaxOn
\newcommand{\ZeroPad}[1]{
	\int_zero_new:N \l_zeropad_i
	\str_clear_new:N \l_zeropad_tmp
	\str_set:Nx \l_zeropad_tmp {#1}
	\int_set:Nn \l_zeropad_i {\str_count:N \l_zeropad_tmp}
	\int_while_do:nn
	{\l_zeropad_i < 3}
	{
		\int_incr:N \l_zeropad_i
		\str_put_left:Nn \l_zeropad_tmp {0}
	}
	\l_zeropad_tmp
}
\ExplSyntaxOff
%<<<

% Setting custom footnote with marks>>>
% ----------------------------------
% \ExplSyntaxOn
% \seq_new:N \jst_footnotes
% \bool_new:N \jst_first_fn
% \bool_set_true:N \jst_first_fn
% \newcommand{\currentfootnotemark}{%
% 	\textsuperscript{\arabic{jstfootnote}}}
%
% \renewcommand{\footnotemark}{%
% 	\stepcounter{jstfootnote}%
% 	\currentfootnotemark%
% }
% \cs_new:Npn \jst_format_footnotes {
% 	\begin{enumerate}[nosep, label=\textsuperscript{\arabic*}]
% 		\seq_map_variable:NNn \jst_footnotes \fntext {
% 		\item \fntext
% 		      }
% 	\end{enumerate}
% }
%
% \renewcommand{\footnote}[1]{\footnotemark%
% 	\seq_put_right:Nn \jst_footnotes {#1}
% 	\begin{jstfootnote*}[!b]
% 		\begingroup
% 		\bool_if:NT \jst_first_fn {
% 			\bool_gset_false:N \jst_first_fn
% 			\rule{0.33\textwidth}{0.5pt}\\
% 		}
% 		\vspace*{-4bp}
% 		\fontsize{9bp}{9bp}\selectfont%
% 		\currentfootnotemark~#1
% 		\endgroup
% 	\end{jstfootnote*}%
% }
% \ExplSyntaxOff
%<<<

% Page/text layout>>>
% =====================
% See: https://www.overleaf.com/learn/latex/Page_size_and_margins
% \RequirePackage[nodisplayskipstretch]{setspace}
\RequirePackage[
	a4paper, inner=2.5cm, outer=2cm, top=3cm, bottom=2cm,
	% Half bottom margin - 1/2 character height
	% minus 1 em because of the headsep
	% footskip=\the\dimexpr 1cm - 1.5em \relax,
	footskip=12pt,
	includefoot=true,
	headheight=1.27cm, headsep=1em,
	marginparwidth=0pt, marginparsep=0pt,
	heightrounded,
]{geometry}
\RequirePackage{indentfirst}
\setlength{\parindent}{0.5cm}
\setlength{\parskip}{6pt plus 3pt minus 3pt}
\setlength{\columnsep}{0.7cm}
\RequirePackage[hang, flushmargin]{footmisc} % Remove footnote indent
%<<<

% Enum spacing>>>
% ------------
\RequirePackage{enumitem}
\setlist{
	noitemsep, nosep,
	parsep=\parskip,
	left=0pt .. \parindent,
	labelwidth=!,
	align=left}
\setlist[enumerate, 1]{label=\arabic*)}
%<<<

% Other dependencies>>>
% ==================
\RequirePackage{xcolor}
\PassOptionsToPackage{
	backend=bibtex,
	defernumbers=true,
	sorting=none,
	citestyle=numeric-comp,
	bibstyle=jstarticle,
	giveninits=true,
	maxcitenames=1,
}{biblatex}
\RequirePackage{biblatex}
\renewcommand*{\bibfont}{\fontsize{9}{11}\selectfont}
\renewcommand{\bibhang}{\parindent}
\renewcommand{\bibparsep}{\parskip}
\renewcommand{\bibitemsep}{\parskip}

% Natbib-like citet and citep
% This was done to be compatible with other journal formats
\ExplSyntaxOn
\seq_new:N \g_jst_cite_list
\int_new:N \g_jst_cite_count
\int_new:N \g_jst_cite_i
\NewDocumentCommand\citep{O{see~}O{~and~}m}{%
\seq_set_from_clist:Nn \g_jst_cite_list {#3}
\int_set:Nn \g_jst_cite_i {0}
\int_set:Nn
\g_jst_cite_count
{\seq_count:N \g_jst_cite_list}
\int_set:Nn \g_jst_cite_count {\g_jst_cite_count - 1}

\mkbibparens{#1
\seq_map_variable:NNn
\g_jst_cite_list
\l_key
{
\int_incr:N \g_jst_cite_i
\cite{\l_key}
\int_compare:nNnT
{\g_jst_cite_i} = {\g_jst_cite_count}
{#2}
\int_compare:nNnT
{\g_jst_cite_i} < {\g_jst_cite_count}
	{,~}
}
}
}
\NewDocumentCommand{\citet}{O{and}m}{
\seq_set_from_clist:Nn \g_jst_cite_list {#2}
\int_set:Nn \g_jst_cite_i {0}
\int_set:Nn
\g_jst_cite_count
{\seq_count:N \g_jst_cite_list}
\int_set:Nn \g_jst_cite_count {\g_jst_cite_count - 1}

\seq_map_variable:NNn
\g_jst_cite_list
\l_key
{
\int_incr:N \g_jst_cite_i
\citeauthor{\l_key}~\cite{\l_key}
\int_compare:nNnT
{\g_jst_cite_i} = {\g_jst_cite_count}
{~#1~}
\int_compare:nNnT
{ \g_jst_cite_i} < {\g_jst_cite_count}
	{,~}
}
}
\ExplSyntaxOff


% Disabling hypenation and using sloppy breaks everything
% Instead, just reduce the hypenation
% https://tex.stackexchange.com/questions/31301/how-to-reduce-the-number-of-hyphenation
\pretolerance=9000
\tolerance=9000
\righthyphenmin=4
\lefthyphenmin=4
\hyphenpenalty=10000
\exhyphenpenalty=9000
\emergencystretch=1.5em
\frenchspacing
\def\nohyphen{%
	\hyphenpenalty=99999%
	\exhyphenpenalty=99999%
	\emergencystretch=3em%
}

% No standalone lines
% See:
% https://tex.stackexchange.com/questions/21983/how-to-avoid-page-breaks-inside-paragraphs
\widowpenalties 5 10000 10000 100 100 0
\clubpenalties 5 10000 10000 100 100 0
\raggedbottom
% \clubpenalty=19996
% \widowpenalty=19999
% \brokenpenalty=4991
% \predisplaypenalty=10000
% \postdisplaypenalty=1549
% \displaywidowpenalty=1602
%<<<

% Frontmatter extras for TOC generation >>>
% =====================================
% Sequences
\ExplSyntaxOn
\seq_new:N \article_titles
\seq_new:N \article_pages
\seq_new:N \article_ids
\seq_new:N \author_belongs
\seq_new:N \author_ids
\seq_new:N \note_ids
\seq_new:N \note_belongs

% Counter
\int_zero_new:N \note_count
\int_zero_new:N \author_count
\int_zero_new:N \article_count

% Special commands for toc file
\newcommand{\nextarticle}{
	\int_incr:N \article_count
	\seq_put_right:Nx \article_ids {\int_use:N \article_count}
}
\newcommand{\tocpage}[1]{\seq_put_right:Nn \article_pages {#1}}

\ExplSyntaxOff
%<<<

% Journal exclusive frontmatter>>>
\ExplSyntaxOn
% Personel
\seq_new:N \editors
\seq_new:N \advisors
\seq_new:N \chiefs
\seq_new:N \supporters
\seq_new:N \guidelines

% Strings
\str_new:N \insertpissn \str_set:Nn \insertpissn {3093-3285}
\str_new:N \inserteissn \str_set:Nn \inserteissn {3093-3315}
\str_new:N \insertaddress
\str_new:N \insertemail
\str_new:N \insertphone
\str_new:N \insertwebsite
\str_new:N \insertjournal
\str_new:N \insertlongjournal
\str_new:N \insertshortjournal
\str_new:N \insertuniversity
\str_new:N \insertsubjournal
\str_new:N \insertvolume
\str_new:N \insertissue

% Setter
\newcommand{\pissn}[1]{\str_gset:Nn \insertpissn {#1}}
\newcommand{\eissn}[1]{\str_gset:Nn \inserteissn {#1}}
\newcommand{\editor}[1]{\seq_put_right:Nn \editors {#1}}
\newcommand{\advisor}[1]{\seq_put_right:Nn \advisors {#1}}
\newcommand{\chief}[1]{\seq_put_right:Nn \chiefs {#1}}
\newcommand{\supporter}[1]{\seq_put_right:Nn \supporters {#1}}
\newcommand{\guideline}[1]{\seq_put_right:Nn \guidelines {#1}}
\newcommand{\volume}[1]{\str_gset:Nx \insertvolume {#1}}
\newcommand{\issue}[1]{\str_set:Nx \insertissue {#1}}
\newcommand{\email}[1]{\str_set:Nx \insertemail {#1}}
\newcommand{\address}[1]{\str_set:Nx \insertaddress {#1}}
\newcommand{\phone}[1]{\str_set:Nx \insertphone {#1}}
\newcommand{\website}[1]{\str_set:Nx \insertwebsite {#1}}
\newcommand{\journal}[1]{\str_set:Nx \insertjournal {#1}}
\newcommand{\subjournal}[1]{\str_set:Nx \insertsubjournal {#1}}
\newcommand{\shortjournal}[1]{\str_set:Nx \insertshortjournal {#1}}
\newcommand{\longjournal}[1]{\str_set:Nx \insertlongjournal {#1}}
\newcommand{\university}[1]{\str_set:Nx \insertuniversity {#1}}

\NewDocumentCommand{\submitTo}{m}{
	\str_case:nnF {#1} {
		{ssad} {
				\pissn{3093-3285}
                \eissn{3093-3315}
				\journal{Journal\ of\ Science\ and\ Technology}
				\subjournal{Smart~Systems~and~Devices}
				\shortjournal{Journal~of~Science~and~Technology~--~Smart~Systems~and~Devices}
				\longjournal{Journal~of~Science~and~Technology~- Technical~Universities}
			}
			{etsd} {
				\pissn{3093-3242}
                \eissn{3093-3579}
				\journal{Journal~of~Science~and~Technology}
				\subjournal{Engineering~and~Technology~for~Sustainable~Development}
				\shortjournal{Journal~of~Science~and~Technology~--~Engineering~and~Technology~for~Sustainable~Development}
			}
			{ds-ai} {
				\pissn{3093-3331}
                \eissn{3093-3323}
				\journal{Journal~of~Science~and~Technology}
				\subjournal{Data~Science~and~Artificial~Intelligence}
				\shortjournal{Journal~of~Science~and~Technology~--~Data~Science~and~Artificial~Intelligence}
			}
			{vht} {
				\pissn{----}
                \eissn{----}
				\journal{Viettel\ High\ Technology\ Report}
				\subjournal{Research\ and\ Development}
				\shortjournal{Viettel~High~Technology~Report}
				\longjournal{Viettel~High~Technology~Report}
			}
	}
	{
		\msg_error:nn {jstarticle} {
			Invalid~submit-to~value~"#1",~the~valid~choices~are:~"ssad",~"etsd",~"ds-ai"~and~"vht"}
	}
}
\ExplSyntaxOff
\submitTo{ssad}
%<<<

% Frontmatter definition >>>
\let\title\undefined
\let\abstract\undefined
\let\endabstract\undefined
\let\author\undefined

\ExplSyntaxOn
% Field definition
\providecommand{\insertabstract}{No abstract}
\cs_new:Npn \inserttitle {}
\str_new:N \insertdoi \str_set:Nn \insertdoi {https://doi.org/10.51316/jst.xxx.ssad.xxxx.xx.x.x}
\str_new:N \insertreceiveddate \str_set:Nn \insertreceiveddate {......./..../......}
\str_new:N \insertaccepteddate \str_set:Nn \insertaccepteddate {......./..../......}
\str_new:N \insertreviseddate \str_set:Nn \insertreviseddate {......./..../......}
\str_new:N \insertonlinedate \str_set:Nn \insertonlinedate {......./..../......}
\int_zero_new:N \pageoffsetnumber
\int_set:Nn \pageoffsetnumber {1}

\seq_new:N \keywords
\seq_new:N \author_names
\seq_new:N \author_marks
\seq_new:N \note_marks
\seq_new:N \note_texts

% Setters
\newcommand{\receiveddate}[1]{\str_set:Nx \insertreceiveddate {#1}}
\newcommand{\accepteddate}[1]{\str_set:Nx \insertaccepteddate {#1}}
\newcommand{\reviseddate}[1]{\str_set:Nx \insertreviseddate {#1}}
\newcommand{\onlinedate}[1]{\str_set:Nx \insertonlinedate {#1}}
\newcommand{\pageoffset}[1]{\int_set:Nn \pageoffsetnumber {#1}}
\newcommand{\doi}[1]{\str_set:Nx \insertdoi {#1}}
\newcommand{\keyword}[1]{\seq_put_right:Nx \keywords {#1}}
\NewDocumentEnvironment{abstract}{+b}{\global\def\insertabstract{#1}}{}
\renewcommand{\abstractname}{\bfseries\rmfamily Abstract}

\NewDocumentCommand{\title}{m}{
	\cs_gset:Npn \inserttitle {#1}

	% TOC interopt
	\seq_put_right:Nn \article_titles {#1}
}

\newcommand{\author}[2][]{
	\str_set:Nx \l_tmpa_str {#1}
	\str_remove_all:Nn \l_tmpa_str {~}
	\seq_put_right:Nx \author_marks {\l_tmpa_str}
	\seq_put_right:Nn \author_names {#2}

	% TOC interopt
	\int_incr:N \author_count
	\seq_put_right:Nx \author_belongs {\int_use:N \article_count}
	\seq_put_right:Nx \author_ids {\int_use:N \author_count}
}

\newcommand{\authornote}[2]{
	\str_set:Nx \l_tmpa_str {#1}
	\str_remove_all:Nn \l_tmpa_str {~}
	\seq_put_right:Nx \note_marks {\l_tmpa_str}
	\seq_put_right:Nn \note_texts {#2}

	% TOC interopt
	\int_incr:N \note_count
	\seq_put_right:Nx \note_belongs {\int_use:N \article_count}
	\seq_put_right:Nx \note_ids {\int_use:N \note_count}
}

% Printings/Getters
\NewDocumentCommand{\printkeywords}{}{
	\seq_gsort:cn {keywords} {
		\str_compare:nNnTF {##1} > {##2}
			{\sort_return_swapped:}
			{\sort_return_same:}
	}
	\seq_use:Nn \keywords {,~}.
}

\newcommand{\printauthors}{
	\int_set:Nn \l_tmpa_int {\seq_count:N \author_names}
	\seq_map_indexed_inline:Nn \author_names {
		\begingroup
		\FormatAuthor{##2}
		\FormatNote{\seq_item:Nn \author_marks {##1}}
		\int_compare:nTF {##1 < \l_tmpa_int} {,\space} {}
		\endgroup
	}
}

\NewDocumentCommand{\PrintPdfAuthors}{}{
	\seq_use:Nnnn \author_names {and} {,} {.}
}

\newcommand{\printnotes}{
	\int_set:Nn \l_tmpa_int {\seq_count:N \note_marks}
	\seq_map_indexed_inline:Nn \note_texts {
		\begingroup
		\FormatNote{\seq_item:Nn \note_marks {##1}}
		\FormatAuthor{\seq_item:Nn \note_texts {##1}}
		\int_compare:nTF {##1 < \l_tmpa_int} {\\} {}
		\endgroup
	}
}

% Formatter
\newcommand{\FormatAuthor}[1]{\textit{#1}}
\newcommand{\FormatNote}[1]{\textsuperscript{\FormatAuthor{#1}}}
\ExplSyntaxOff
%<<<

% Header and footer>>>
% =================
\RequirePackage{fancyhdr}
\ExplSyntaxOn

% For the article pages
\fancypagestyle{jst}{
	\fancyhead[lr]{} % No header on left/right
	\fancyfoot[lr]{} % No footer on left/right
	\fancyfoot[c]{\vfill\thepage\vfill} % Page number at the center
	\fancyhead[c]{} % Hide journal running header
	\renewcommand{\headrulewidth}{0pt} % Remove top horizontal line
}

% For the table of content only
\fancypagestyle{jst-toc}{
	\fancyhead[lr]{} % No header on left/right
	\fancyfoot{} % No footer
	\fancyhead[c]{} % Hide TOC running header
	\renewcommand{\headrulewidth}{0pt} % Remove top horizontal line
}
\pagestyle{jst}
\ExplSyntaxOff

% Add a hook for multicolumn at the begin and end document
% Also, titlepage does not have column
\ExplSyntaxOn
% \AddToHook{begindocument/end}{\begin{multicols*}{2}}
% 		\AddToHook{enddocument}{\end{multicols*}}
% \AddToHook{env/titlepage/after}{\begin{multicols*}{2}}
% 		\AddToHook{env/titlepage/before}{\end{multicols*}}
\ExplSyntaxOff

% Redefine titlepage
% ==================
\makeatletter
\providecommand{\abstractname}{Abstract}
\providecommand{\keywordname}{Keywords:}
\renewcommand{\maketitle}{%
	\twocolumn[

		% Make the PDF use the provided information as properties
		% =======================================================
		% \hypersetup{%
		% 	pdftitle = {\inserttitle},%
		% 	pdfkeywords = {\printkeywords},%
		% 	pdfauthor = {\PrintPdfAuthors},%
		% }

		\null\hfill%
		\begin{minipage}{\dimexpr\textwidth - 0.5cm}
			\setlength{\parindent}{0pt}
			\setlength{\parskip}{0pt}
			%
			% Switch to one column
			% --------------------
			% Reference: https://tex.stackexchange.com/questions/91137/preventing-page-break-in-documentclasstwocolumnarticle-after-author
			% Update:
			% \twocolumn is now not used because we want spanning footnote, which
			% requires either rewriting deep diving into latex internals, or just
			% using multicol packages.
			% I will just use the multicol package. There's no time.

			% Print title, authors and affiliations
			% =====================================
			\vbox\bgroup
			\centering
			\vspace*{0.5em}
			{\titleFont\inserttitle}
			\vspace*{1em}\\
			{\authorFont\printauthors}
			\vspace*{3pt}\\
			{\affiliationFont\printnotes}
			\egroup

			\vspace*{6pt}

			% Insert the abstract and keywords
			% --------------------------------
			% The line break instead of \par is intended
			\begingroup
			\bfseries\fontsize{9bp}{9bp}\sffamily
			\textbf{\abstractname}
			\endgroup
			\par
			\noindent\begingroup
			\abstractFont\insertabstract
			\endgroup

			\vspace*{6bp}

			\begingroup
			\edef\printkeywordsnow{\printkeywords}%
			\keywordFont\keywordname
			\space\printkeywordsnow
			\endgroup

		\end{minipage}%

		% Switch to back to two column
		% ----------------------------
		% Nothing to be done, really
		% Just add a little spacing on the way out
		% Add some glue for better spacing on the first page
		\vspace*{12pt plus 24pt minus 11pt}%
	]

	% Metadata footer (ISSN/DOI/dates) intentionally hidden.
	% \begin{multicols*}{2}
}
\makeatother
%<<<

% Format headings and table captions>>>
% ==================================
% Docs: https://ctan.org/pkg/titlesec?lang=en
% Docs: https://ctan.org/pkg/caption?lang=en
% Docs: https://ctan.org/pkg/float?lang=en
% Quick references:
% - titlespacing: left-before-after-right
% - titleformat: \titleformat{⟨command ⟩}[⟨shape⟩]{⟨format⟩}{⟨label ⟩}{⟨sep⟩}{⟨before-code⟩}[⟨after-code⟩]
\RequirePackage{titlesec}
\RequirePackage{float}
\RequirePackage[
	labelsep=period,
	% format=plain,
	skip=0pt plus 2pt minus 0pt,
	% This is the position on the LINE, not in the float
	% figureposition=top,
	% tableposition=bottom, % 
	% Table/figure caption will be justified if
	% there are multiple lines, but centered if
	% is it single line.
	justification=justified,
	singlelinecheck=true,
]{caption}
\captionsetup[table]{
    % Table caption goes left, regardless of single line
    % or not.
    justification=justified,
    singlelinecheck=false
}
\abovecaptionskip=6pt plus 2pt minus 2pt
\belowcaptionskip=6pt plus 2pt minus 2pt
\newfloat{jstfootnote}{bp}{jstfootnote}
\floatstyle{plaintop}\restylefloat{table}
\floatstyle{plain}\restylefloat{figure}
\titlespacing*{\section}{0pt}{3pt plus 3pt minus 3pt}{0pt}
\titlespacing*{\subsection}{0pt}{0pt plus 3pt}{0pt}
\titlespacing*{\subsubsection}{0pt}{0pt plus 3pt}{0pt}
\titleformat{\section}{\headingAFont\nohyphen}{\thesection.\space}{0pt plus 6pt}{}
\titleformat{\subsection}{\headingBFont\nohyphen}{\thesubsection.\space}{0pt plus 6pt}{}
\titleformat{\subsubsection}{\headingCFont\nohyphen}{\thesubsubsection.\space}{0pt plus 6pt}{}
\AtBeginDocument{% Must use hook, incase the user load babel
	\renewcommand{\figurename}{Fig.}%
	\renewcommand{\tablename}{Table}}

% https://aty.sdsu.edu/bibliog/latex/floats.html
% Alter some LaTeX defaults for better treatment of figures:
% See p.105 of "TeX Unbound" for suggested values.
% See pp. 199-200 of Lamport's "LaTeX" book for details.
% General parameters, for ALL pages:
\renewcommand{\topfraction}{0.9}	% max fraction of floats at top
\renewcommand{\bottomfraction}{0.8}	% max fraction of floats at bottom
% Parameters for TEXT pages (not float pages):
\setcounter{topnumber}{2}
\setcounter{bottomnumber}{2}
\setcounter{totalnumber}{4}     % 2 may work better
\setcounter{dbltopnumber}{2}    % for 2-column pages
\renewcommand{\dbltopfraction}{0.9}	% fit big float above 2-col. text
\renewcommand{\textfraction}{0.07}	% allow minimal text w. figs
% Parameters for FLOAT pages (not text pages):
\renewcommand{\floatpagefraction}{0.7}	% require fuller float pages
% N.B.: floatpagefraction MUST be less than topfraction !!
\renewcommand{\dblfloatpagefraction}{0.7}	% require fuller float pages

% Reduce float spacing
% https://tex.stackexchange.com/questions/62720/vertical-space-after-algorithm
% https://tex.stackexchange.com/questions/26521/how-to-change-the-spacing-between-figures-tables-and-text
% Yes, provide crap tons of spacing just to make the
% layout looks pretty
\setlength{\intextsep}{\parskip}
\setlength{\floatsep}{\parskip}
\setlength{\textfloatsep}{\parskip}
\setlength{\dbltextfloatsep}{10pt plus 10pt minus 10pt}
\setlength{\dblfloatsep}{10pt plus 10pt minus 10pt}

% Rework all the heading commands
% Starred variant is required for biblatex...
% Since we requires every section to have numbering
% We will just define the same behaviour to all of them
% https://tex.stackexchange.com/questions/376375/using-ifstar-to-define-a-star-variant
% \makeatletter
% \newcommand\jst@startsection[3]{
% 	\par
% 	\refstepcounter{#1}%
% 	\addcontentsline{toc}{#1}{#3}%
% 	\begingroup%
% 	#2
% 	\noindent\protect{\csname the#1\endcsname}%
% 	.~#3%
% 	\endgroup%
% 	\par%
% }
% \newcommand\JstStartSection[2]{%
% 	\@ifstar
% 	{\jst@startsection{#1}{#2}}
% 	{\jst@startsection{#1}{#2}}}
% \renewcommand\section{%
%   \addpenalty{-400}%
%   \JstStartSection{section}{\bfseries}}
% \renewcommand\subsection{%
% 	\addpenalty{-200}%
% 	\JstStartSection{subsection}{\bfseries\itshape}}
% \renewcommand\subsubsection{%
% 	\JstStartSection{subsubsection}{\itshape}}

% https://tex.stackexchange.com/questions/60477/remove-space-after-figure-and-before-text
% \newcommand{\squeezeup}{\vspace{-2.5mm}}
%<<<

% Math setups>>>
% ===========
% What is going on:
% - Load some commonly used packages (or useful ones)
% - Remove the vertical spacing of displays 
% - Fix the spacing on \left, \right
% See:
% - https://tex.stackexchange.com/questions/224987/how-to-decrease-space-above-and-below-displayed-equations
% - https://tex.stackexchange.com/questions/2607/spacing-around-left-and-right
\setlength{\abovedisplayskip}{0pt plus 6pt minus 6pt}
\setlength{\abovedisplayshortskip}{0pt plus 3pt minus 3pt}
\setlength{\belowdisplayskip}{0pt plus 6pt minus 6pt}
\setlength{\belowdisplayshortskip}{0pt plus 3pt minus 3pt}
\let\originalleft\left
\let\originalright\right
\renewcommand{\left}{\mathopen{}\mathclose\bgroup\originalleft}
\renewcommand{\right}{\aftergroup\egroup\originalright}
%<<<

% Inter-opteration using data override file>>>
\IfFileExists{jst-data-override.tex}{\input{jst-data-override.tex}}{\relax}
%<<<

% Journal exclusive pages (general guidelines, rules, toc)>>>
\ExplSyntaxOn
\newcommand{\MakeSubcover}{%
	\begingroup
	\begin{titlepage}
		\sffamily
		% Logo and headings

		\begin{center}
			\includegraphics[width=0.5\textwidth]{jst-logo}
			\\\vfill

			{\fontsize{10}{10}\selectfont\insertjournal}
			\\\smallskip

			{\bfseries\fontsize{16}{16}\selectfont\insertsubjournal}
			\\\smallskip

			{
				\fontsize{10}{10}\selectfont
				Volume\ \insertvolume,\space
				Issue\ \insertissue,\space
				\insertdate.
			}
		\end{center}
		\vfill

		% Personel list
		% =============
		\begingroup

		\fontsize{8}{10}\selectfont
		\setbox0\vbox{
			\textbf{Editorial\ Board}\\
			\seq_use:Nn \editors {\\}
		}

		\begin{minipage}{0.475\textwidth}
			\vbox to \ht0 {
				\textbf{Deputy\ Editor\ in\ Chief}\\
				\seq_use:Nn \chiefs {\\}
				\vfill

				\textbf{Advisory\ Board}\\
				\seq_use:Nn \advisors {\\}
			}
		\end{minipage}
		\hfill
		\begin{minipage}{0.475\textwidth}%
			\textbf{Editorial\ Board}\\
			\seq_use:Nn \editors {\\}
		\end{minipage}

		% Supporter
		\vfill\vfill

		\begin{center}
			\textbf{Supported\ by}\\
			\seq_use:Nn \supporters {\\}
		\end{center}

		\endgroup
	\end{titlepage}
	\endgroup
}

\newcommand{\MakeGuidelines}{
	\begingroup
	\begin{titlepage}
		\vbox to \textheight {
			\sffamily{}
			\fontsize{8}{10}\selectfont
			\centering
			\vfill

			\textbf{GENERAL\ GUIDELINES}
			\par\bigskip

			\begin{minipage}{0.8\textwidth}
				\begingroup
				\begin{enumerate}[label=\arabic*.]
					\setlength{\itemsep}{4pt}
					\seq_map_variable:NNn \guidelines \l_guideline {\item \l_guideline}
				\end{enumerate}
				\endgroup
			\end{minipage}

			\vfill
			\textbf{\insertlongjournal}\\
			\textbf{\insertuniversity}\\
			\insertaddress\\
			Tel:\space\insertphone
			\space\textbar\space
			email:\space\insertemail
			\space\textbar\space
			website:\space\insertwebsite

		}
	\end{titlepage}
	\endgroup
}

\seq_new:N \l_toc_ids
\seq_new:N \l_toc_authors
\seq_new:N \l_toc_notes
\seq_new:N \l_toc_entries
\str_new:N \l_toc_article_page
\str_new:N \l_toc_article_title
\newcommand{\MakeToc}{
	\seq_clear:N \l_toc_ids
	\seq_clear:N \l_toc_authors
	\seq_clear:N \l_toc_notes
	\seq_clear:N \l_toc_entries

	\begin{titlepage}
		\thispagestyle{jst-toc}

		% Headline
		\begin{center}
			\textbf{Contents}
		\end{center}
		\par\bigskip


		% Loop through articles and format each articles
		\enumerate[label=\arabic*.]
		\seq_map_variable:NNn \article_ids \article_id {
			% Get article page and title
			\str_set:Nx \l_toc_article_title {\seq_item:Nn \article_titles {\article_id}}
			\str_set:Nx \l_toc_article_page {\seq_item:Nn \article_pages {\article_id}}

			% Get list of authors
			\seq_set_filter:NNn \l_toc_ids \author_ids {
				\int_compare_p:n {
					\seq_item:Nn \author_belongs { ##1 } = \article_id
				}
			}
			\seq_set_map:NNn \l_toc_authors \l_toc_ids {
				\FormatAuthor{\seq_item:Nn \author_names {##1}}
				\FormatNote{\seq_item:Nn \author_marks {##1}}
			}

			% Get list of affiliation notes
			\seq_set_filter:NNn \l_toc_ids \note_ids {
				\int_compare_p:n {
					\seq_item:Nn \note_belongs { ##1 } = \article_id
				}
			}
			\seq_set_map:NNn \l_toc_notes \l_toc_ids {
				\FormatNote{\seq_item:Nn \note_marks {##1}}
				\FormatAuthor{\seq_item:Nn \note_texts {##1}}
			}

			% Format information
			\item \l_toc_article_title \hfill \l_toc_article_page\par
			\hspace{0.04\textwidth}\begin{minipage}{0.95\textwidth}
				\smallskip
				\seq_use:Nn \l_toc_authors {,~}\\
				\seq_use:Nn \l_toc_notes {\\}\\
				\smallskip
			\end{minipage}
		}
		\endenumerate
	\end{titlepage}
}

\ExplSyntaxOff
%<<<

% Hyper ref should be loaded last>>>
\RequirePackage{hyperref}
\hypersetup{
	colorlinks,
	linkcolor=black,
	citecolor=black,
	urlcolor=black,
}
\urlstyle{rm}
%<<<
\makeatother
